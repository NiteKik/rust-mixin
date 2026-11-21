//! Build identity and payload checks before LLVM parsing.
use crate::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Input {
    file: String,
    sha256: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    schema: u32,
    rustc: String,
    llvm: [u32; 3],
    triple: String,
    cpu: String,
    lowering_flags: Vec<String>,
    inputs: Vec<Input>,
}

pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

impl Manifest {
    pub fn read_game(root: &Path) -> Result<(Self, Vec<u8>)> {
        let bytes = std::fs::read(root.join("manifest.json")).map_err(|e| e.to_string())?;
        let manifest = Self::parse(&bytes)?;
        let game = manifest.payload(root, "retained_game.bc")?;
        Ok((manifest, game))
    }

    /// Validate a game manifest and its base bitcode embedded in an executable.
    pub fn read_embedded_game(manifest: &[u8], game: &[u8]) -> Result<Self> {
        let manifest = Self::parse(manifest)?;
        manifest.input("retained_game.bc")?.verify(game)?;
        Ok(manifest)
    }

    fn parse(bytes: &[u8]) -> Result<Self> {
        // Windows PowerShell 5 may emit a UTF-8 BOM.
        let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
        let manifest: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if manifest.schema != 1
            || manifest.llvm != [21, 1, 3]
            || manifest.triple != "x86_64-pc-windows-msvc"
            || manifest.cpu != "x86-64"
            || !manifest
                .rustc
                .contains("commit-hash: ded5c06cf21d2b93bffd5d884aa6e96934ee4234")
            || !manifest.rustc.contains("LLVM version: 21.1.3")
        {
            return Err("Compiler/target/manifest identity mismatch".into());
        }
        Ok(manifest)
    }

    pub fn payload(&self, root: &Path, name: &str) -> Result<Vec<u8>> {
        let input = self.input(name)?;
        let bytes = std::fs::read(root.join(name)).map_err(|e| e.to_string())?;
        input.verify(&bytes)?;
        Ok(bytes)
    }

    fn input(&self, name: &str) -> Result<&Input> {
        let mut entries = self.inputs.iter().filter(|input| input.file == name);
        match (entries.next(), entries.next()) {
            (Some(input), None) => Ok(input),
            _ => Err(format!("Expected exactly one {name}")),
        }
    }
}

impl Input {
    fn verify(&self, bytes: &[u8]) -> Result<()> {
        if sha256(bytes) != self.sha256 {
            return Err(format!("Bitcode hash mismatch: {}", self.file));
        }
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModManifest {
    schema: u32,
    rustc_commit: String,
    llvm: [u32; 3],
    triple: String,
    cpu: String,
    expected_game_sha256: String,
    code_sha256: String,
}

impl ModManifest {
    pub fn read(directory: &Path, game: &[u8]) -> Result<Vec<u8>> {
        let bytes = std::fs::read(directory.join("mod.json")).map_err(|e| e.to_string())?;
        let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes);
        let manifest: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if manifest.schema != 1
            || manifest.rustc_commit != "ded5c06cf21d2b93bffd5d884aa6e96934ee4234"
            || manifest.llvm != [21, 1, 3]
            || manifest.triple != "x86_64-pc-windows-msvc"
            || manifest.cpu != "x86-64"
            || manifest.expected_game_sha256 != sha256(game)
        {
            return Err("mod compiler/game identity mismatch".into());
        }
        let code = std::fs::read(directory.join("code.bc")).map_err(|e| e.to_string())?;
        if manifest.code_sha256 != sha256(&code) {
            return Err("mod payload hash mismatch".into());
        }
        Ok(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests_support::read_fixture;
    fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
        std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).map_err(|e| e.to_string())
    }

    #[test]
    fn manifest_rejects_changed_payload_and_compiler_identity_before_llvm_parsing() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let source = root.join("artifacts");
        let (mut manifest, game, handler) = read_fixture(&source).unwrap();
        let scratch = root
            .join("target")
            .join(format!("manifest-test-{}", std::process::id()));
        std::fs::create_dir_all(&scratch).unwrap();
        std::fs::write(scratch.join("retained_game.bc"), &game).unwrap();
        std::fs::write(scratch.join("retained_mod.bc"), &handler).unwrap();
        write_json(&scratch.join("manifest.json"), &manifest).unwrap();
        assert!(read_fixture(&scratch).is_ok());
        let manifest_bytes = serde_json::to_vec(&manifest).unwrap();
        assert!(Manifest::read_embedded_game(&manifest_bytes, &game).is_ok());
        assert!(matches!(
            Manifest::read_embedded_game(&manifest_bytes, b"changed"),
            Err(error) if error.contains("hash mismatch")
        ));
        std::fs::write(scratch.join("retained_game.bc"), b"changed").unwrap();
        assert!(
            read_fixture(&scratch)
                .err()
                .unwrap()
                .contains("hash mismatch")
        );
        std::fs::write(scratch.join("retained_game.bc"), &game).unwrap();
        manifest.llvm = [19, 1, 5];
        write_json(&scratch.join("manifest.json"), &manifest).unwrap();
        assert!(
            read_fixture(&scratch)
                .err()
                .unwrap()
                .contains("identity mismatch")
        );
        std::fs::remove_dir_all(scratch).unwrap();
    }

    #[test]
    fn external_mod_rejects_wrong_game_and_changed_code() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let (_, game, handler) = read_fixture(&root.join("artifacts")).unwrap();
        let scratch = root
            .join("target")
            .join(format!("external-manifest-test-{}", std::process::id()));
        std::fs::create_dir_all(&scratch).unwrap();
        std::fs::write(scratch.join("code.bc"), &handler).unwrap();
        let mut manifest = ModManifest {
            schema: 1,
            rustc_commit: "ded5c06cf21d2b93bffd5d884aa6e96934ee4234".into(),
            llvm: [21, 1, 3],
            triple: "x86_64-pc-windows-msvc".into(),
            cpu: "x86-64".into(),
            expected_game_sha256: sha256(&game),
            code_sha256: sha256(&handler),
        };
        write_json(&scratch.join("mod.json"), &manifest).unwrap();
        assert_eq!(ModManifest::read(&scratch, &game).unwrap(), handler);
        std::fs::write(scratch.join("code.bc"), b"changed").unwrap();
        assert!(
            ModManifest::read(&scratch, &game)
                .unwrap_err()
                .contains("hash mismatch")
        );
        std::fs::write(scratch.join("code.bc"), &handler).unwrap();
        manifest.expected_game_sha256 = "wrong".into();
        write_json(&scratch.join("mod.json"), &manifest).unwrap();
        assert!(
            ModManifest::read(&scratch, &game)
                .unwrap_err()
                .contains("identity mismatch")
        );
        std::fs::remove_dir_all(scratch).unwrap();
    }
}
