; ModuleID = 'retained-rust'
source_filename = "llvm_fixture.45319190217dc304-cgu.0"
target datalayout = "e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-windows-msvc"

@__rsmx_meta_59aed09041305b4e1e46c395 = constant [371 x i8] c"{\22arity\22:2,\22entry\22:\22__rsmx_entry_08aa4a1cf4c3009dca5ed38f\22,\22kind\22:\22target\22,\22logical_id\22:\22demo::combat::apply_damage\22,\22retention_noinline\22:true,\22return_hook\22:{\22body\22:\22__rsmx_body_08aa4a1cf4c3009dca5ed38f\22,\22symbol\22:\22__rsmx_return_08aa4a1cf4c3009dca5ed38f\22,\22type_probe\22:\22__rsmx_target_type_08aa4a1cf4c3009dca5ed38f\22},\22schema\22:1,\22symbol\22:\22__rsmx_fn_08aa4a1cf4c3009dca5ed38f\22}", align 1
@__rsmx_meta_794813aff581a360f1387746 = constant [371 x i8] c"{\22arity\22:2,\22entry\22:\22__rsmx_entry_6c5480d415ed2b55da7ba335\22,\22kind\22:\22target\22,\22logical_id\22:\22demo::combat::armor_reduce\22,\22retention_noinline\22:true,\22return_hook\22:{\22body\22:\22__rsmx_body_6c5480d415ed2b55da7ba335\22,\22symbol\22:\22__rsmx_return_6c5480d415ed2b55da7ba335\22,\22type_probe\22:\22__rsmx_target_type_6c5480d415ed2b55da7ba335\22},\22schema\22:1,\22symbol\22:\22__rsmx_fn_6c5480d415ed2b55da7ba335\22}", align 1
@__rsmx_meta_826f26bacc73c401cf7b9a6e = constant [367 x i8] c"{\22arity\22:2,\22entry\22:\22__rsmx_entry_3cb4fc20c82e364a35b1be9f\22,\22kind\22:\22target\22,\22logical_id\22:\22demo::combat::two_hits\22,\22retention_noinline\22:true,\22return_hook\22:{\22body\22:\22__rsmx_body_3cb4fc20c82e364a35b1be9f\22,\22symbol\22:\22__rsmx_return_3cb4fc20c82e364a35b1be9f\22,\22type_probe\22:\22__rsmx_target_type_3cb4fc20c82e364a35b1be9f\22},\22schema\22:1,\22symbol\22:\22__rsmx_fn_3cb4fc20c82e364a35b1be9f\22}", align 1
@__rsmx_meta_90db65816e2f8ab9db351e3a = constant [365 x i8] c"{\22arity\22:2,\22entry\22:\22__rsmx_entry_e02ec5709ce5f1a52a6d8b6f\22,\22kind\22:\22target\22,\22logical_id\22:\22demo::combat::caller\22,\22retention_noinline\22:true,\22return_hook\22:{\22body\22:\22__rsmx_body_e02ec5709ce5f1a52a6d8b6f\22,\22symbol\22:\22__rsmx_return_e02ec5709ce5f1a52a6d8b6f\22,\22type_probe\22:\22__rsmx_target_type_e02ec5709ce5f1a52a6d8b6f\22},\22schema\22:1,\22symbol\22:\22__rsmx_fn_e02ec5709ce5f1a52a6d8b6f\22}", align 1
@__rsmx_meta_c2db6e1db5f0eccb533140aa = constant [331 x i8] c"{\22callee\22:null,\22id\22:\22minimum_damage\22,\22kind\22:\22modify_return_result\22,\22operation\22:\22modify_return_result\22,\22ordinal\22:0,\22parameter\22:null,\22priority\22:0,\22require_matches\22:1,\22return_type_probe\22:\22__rsmx_handler_type_16c5b70e7fc8c074fa7e8ca9\22,\22schema\22:3,\22symbol\22:\22__rsmx_handler_16c5b70e7fc8c074fa7e8ca9\22,\22target\22:\22demo::combat::apply_damage\22}", align 1
@anon.0639e836b9e1b73c458d36206545f83f.0 = private unnamed_addr constant <{ ptr, ptr }> <{ ptr inttoptr (i64 794668071241239420 to ptr), ptr inttoptr (i64 -4297518438299490226 to ptr) }>, align 8
@llvm.used = appending global [5 x ptr] [ptr @__rsmx_meta_59aed09041305b4e1e46c395, ptr @__rsmx_meta_794813aff581a360f1387746, ptr @__rsmx_meta_826f26bacc73c401cf7b9a6e, ptr @__rsmx_meta_90db65816e2f8ab9db351e3a, ptr @__rsmx_meta_c2db6e1db5f0eccb533140aa], section "llvm.metadata"

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @_ZN12llvm_fixture14fixture_damage17hef4468a5d7332544E(double noundef %amount, double noundef %armor) unnamed_addr #0 {
start:
  %_3.i.i.i.i = fmul double %armor, 5.000000e-01
  %_0.i.i.i.i = fsub double %amount, %_3.i.i.i.i
  %_5.i.i = fcmp olt double %_0.i.i.i.i, 1.800000e+01
  %_3.sroa.0.0.i.i = select i1 %_5.i.i, double 1.800000e+01, double %_0.i.i.i.i
  ret double %_3.sroa.0.0.i.i
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @_ZN12llvm_fixture20fixture_armor_reduce17hafa8a93c603f5ad8E(double noundef %amount, double noundef %armor) unnamed_addr #0 {
start:
  %_3.i.i = fmul double %armor, 5.000000e-01
  %_0.i.i = fsub double %amount, %_3.i.i
  ret double %_0.i.i
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @__rsmx_body_08aa4a1cf4c3009dca5ed38f(double noundef %amount, double noundef %armor) unnamed_addr #0 {
start:
  %_3.i.i = fmul double %armor, 5.000000e-01
  %_0.i.i = fsub double %amount, %_3.i.i
  ret double %_0.i.i
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @__rsmx_body_3cb4fc20c82e364a35b1be9f(double noundef %amount, double noundef %armor) unnamed_addr #0 {
start:
  %_3.i.i = fmul double %armor, 5.000000e-01
  %_0.i.i = fsub double %amount, %_3.i.i
  %_5 = fadd double %amount, 2.000000e+00
  %_0.i.i2 = fsub double %_5, %_3.i.i
  %_0 = fadd double %_0.i.i, %_0.i.i2
  ret double %_0
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @__rsmx_body_6c5480d415ed2b55da7ba335(double noundef %amount, double noundef %armor) unnamed_addr #0 {
start:
  %_3 = fmul double %armor, 5.000000e-01
  %_0 = fsub double %amount, %_3
  ret double %_0
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @__rsmx_body_e02ec5709ce5f1a52a6d8b6f(double noundef %amount, double noundef %armor) unnamed_addr #0 {
start:
  %_3.i.i.i.i = fmul double %armor, 5.000000e-01
  %_0.i.i.i.i = fsub double %amount, %_3.i.i.i.i
  %_5.i.i = fcmp olt double %_0.i.i.i.i, 1.800000e+01
  %_3.sroa.0.0.i.i = select i1 %_5.i.i, double 1.800000e+01, double %_0.i.i.i.i
  %_0 = fadd double %_3.sroa.0.0.i.i, 1.000000e+00
  ret double %_0
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @__rsmx_entry_08aa4a1cf4c3009dca5ed38f(double noundef %__arg0, double noundef %__arg1) unnamed_addr #0 {
start:
  %_3.i.i.i.i = fmul double %__arg1, 5.000000e-01
  %_0.i.i.i.i = fsub double %__arg0, %_3.i.i.i.i
  %_5.i.i = fcmp olt double %_0.i.i.i.i, 1.800000e+01
  %_3.sroa.0.0.i.i = select i1 %_5.i.i, double 1.800000e+01, double %_0.i.i.i.i
  ret double %_3.sroa.0.0.i.i
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @__rsmx_entry_3cb4fc20c82e364a35b1be9f(double noundef %__arg0, double noundef %__arg1) unnamed_addr #0 {
start:
  %_3.i.i.i.i = fmul double %__arg1, 5.000000e-01
  %_0.i.i.i.i = fsub double %__arg0, %_3.i.i.i.i
  %_5.i.i = fadd double %__arg0, 2.000000e+00
  %_0.i.i2.i.i = fsub double %_5.i.i, %_3.i.i.i.i
  %_0.i.i = fadd double %_0.i.i.i.i, %_0.i.i2.i.i
  ret double %_0.i.i
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @__rsmx_entry_6c5480d415ed2b55da7ba335(double noundef %__arg0, double noundef %__arg1) unnamed_addr #0 {
start:
  %_3.i.i = fmul double %__arg1, 5.000000e-01
  %_0.i.i = fsub double %__arg0, %_3.i.i
  ret double %_0.i.i
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @__rsmx_entry_e02ec5709ce5f1a52a6d8b6f(double noundef %__arg0, double noundef %__arg1) unnamed_addr #0 {
start:
  %_3.i.i.i.i.i.i = fmul double %__arg1, 5.000000e-01
  %_0.i.i.i.i.i.i = fsub double %__arg0, %_3.i.i.i.i.i.i
  %_5.i.i.i.i = fcmp olt double %_0.i.i.i.i.i.i, 1.800000e+01
  %_3.sroa.0.0.i.i.i.i = select i1 %_5.i.i.i.i, double 1.800000e+01, double %_0.i.i.i.i.i.i
  %_0.i.i = fadd double %_3.sroa.0.0.i.i.i.i, 1.000000e+00
  ret double %_0.i.i
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @__rsmx_fn_08aa4a1cf4c3009dca5ed38f(double noundef %__mixin_input0, double noundef %__mixin_input1) unnamed_addr #0 {
start:
  %_3.i.i.i = fmul double %__mixin_input1, 5.000000e-01
  %_0.i.i.i = fsub double %__mixin_input0, %_3.i.i.i
  %_5.i = fcmp olt double %_0.i.i.i, 1.800000e+01
  %_3.sroa.0.0.i = select i1 %_5.i, double 1.800000e+01, double %_0.i.i.i
  ret double %_3.sroa.0.0.i
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @__rsmx_fn_3cb4fc20c82e364a35b1be9f(double noundef %__mixin_input0, double noundef %__mixin_input1) unnamed_addr #0 {
start:
  %_3.i.i.i = fmul double %__mixin_input1, 5.000000e-01
  %_0.i.i.i = fsub double %__mixin_input0, %_3.i.i.i
  %_5.i = fadd double %__mixin_input0, 2.000000e+00
  %_0.i.i2.i = fsub double %_5.i, %_3.i.i.i
  %_0.i = fadd double %_0.i.i.i, %_0.i.i2.i
  ret double %_0.i
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @__rsmx_fn_6c5480d415ed2b55da7ba335(double noundef %__mixin_input0, double noundef %__mixin_input1) unnamed_addr #0 {
start:
  %_3.i = fmul double %__mixin_input1, 5.000000e-01
  %_0.i = fsub double %__mixin_input0, %_3.i
  ret double %_0.i
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @__rsmx_fn_e02ec5709ce5f1a52a6d8b6f(double noundef %__mixin_input0, double noundef %__mixin_input1) unnamed_addr #0 {
start:
  %_3.i.i.i.i.i = fmul double %__mixin_input1, 5.000000e-01
  %_0.i.i.i.i.i = fsub double %__mixin_input0, %_3.i.i.i.i.i
  %_5.i.i.i = fcmp olt double %_0.i.i.i.i.i, 1.800000e+01
  %_3.sroa.0.0.i.i.i = select i1 %_5.i.i.i, double 1.800000e+01, double %_0.i.i.i.i.i
  %_0.i = fadd double %_3.sroa.0.0.i.i.i, 1.000000e+00
  ret double %_0.i
}

; Function Attrs: noinline nounwind uwtable
define void @__rsmx_return_08aa4a1cf4c3009dca5ed38f(ptr noundef %result) unnamed_addr #1 {
start:
  %0 = alloca [8 x i8], align 8
  call void @llvm.lifetime.start.p0(i64 8, ptr nonnull %0)
  store ptr %result, ptr %0, align 8
  call void asm sideeffect "", "r,~{memory}"(ptr nonnull %0) #6, !srcloc !2
  call void @llvm.lifetime.end.p0(i64 8, ptr nonnull %0)
  ret void
}

; Function Attrs: noinline nounwind uwtable
define void @__rsmx_return_3cb4fc20c82e364a35b1be9f(ptr noundef %result) unnamed_addr #1 {
start:
  %0 = alloca [8 x i8], align 8
  call void @llvm.lifetime.start.p0(i64 8, ptr nonnull %0)
  store ptr %result, ptr %0, align 8
  call void asm sideeffect "", "r,~{memory}"(ptr nonnull %0) #6, !srcloc !2
  call void @llvm.lifetime.end.p0(i64 8, ptr nonnull %0)
  ret void
}

; Function Attrs: noinline nounwind uwtable
define void @__rsmx_return_6c5480d415ed2b55da7ba335(ptr noundef %result) unnamed_addr #1 {
start:
  %0 = alloca [8 x i8], align 8
  call void @llvm.lifetime.start.p0(i64 8, ptr nonnull %0)
  store ptr %result, ptr %0, align 8
  call void asm sideeffect "", "r,~{memory}"(ptr nonnull %0) #6, !srcloc !2
  call void @llvm.lifetime.end.p0(i64 8, ptr nonnull %0)
  ret void
}

; Function Attrs: noinline nounwind uwtable
define void @__rsmx_return_e02ec5709ce5f1a52a6d8b6f(ptr noundef %result) unnamed_addr #1 {
start:
  %0 = alloca [8 x i8], align 8
  call void @llvm.lifetime.start.p0(i64 8, ptr nonnull %0)
  store ptr %result, ptr %0, align 8
  call void asm sideeffect "", "r,~{memory}"(ptr nonnull %0) #6, !srcloc !2
  call void @llvm.lifetime.end.p0(i64 8, ptr nonnull %0)
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: readwrite) uwtable
define void @__rsmx_target_type_08aa4a1cf4c3009dca5ed38f(ptr noundef writeonly captures(none) initializes((0, 16)) %__output) unnamed_addr #2 {
start:
  tail call void @llvm.memcpy.p0.p0.i64(ptr noundef nonnull align 8 dereferenceable(16) %__output, ptr noundef nonnull align 8 dereferenceable(16) @anon.0639e836b9e1b73c458d36206545f83f.0, i64 16, i1 false)
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: readwrite) uwtable
define void @__rsmx_target_type_3cb4fc20c82e364a35b1be9f(ptr noundef writeonly captures(none) initializes((0, 16)) %__output) unnamed_addr #2 {
start:
  tail call void @llvm.memcpy.p0.p0.i64(ptr noundef nonnull align 8 dereferenceable(16) %__output, ptr noundef nonnull align 8 dereferenceable(16) @anon.0639e836b9e1b73c458d36206545f83f.0, i64 16, i1 false)
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: readwrite) uwtable
define void @__rsmx_target_type_6c5480d415ed2b55da7ba335(ptr noundef writeonly captures(none) initializes((0, 16)) %__output) unnamed_addr #2 {
start:
  tail call void @llvm.memcpy.p0.p0.i64(ptr noundef nonnull align 8 dereferenceable(16) %__output, ptr noundef nonnull align 8 dereferenceable(16) @anon.0639e836b9e1b73c458d36206545f83f.0, i64 16, i1 false)
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: readwrite) uwtable
define void @__rsmx_target_type_e02ec5709ce5f1a52a6d8b6f(ptr noundef writeonly captures(none) initializes((0, 16)) %__output) unnamed_addr #2 {
start:
  tail call void @llvm.memcpy.p0.p0.i64(ptr noundef nonnull align 8 dereferenceable(16) %__output, ptr noundef nonnull align 8 dereferenceable(16) @anon.0639e836b9e1b73c458d36206545f83f.0, i64 16, i1 false)
  ret void
}

; Function Attrs: nofree norecurse nosync nounwind memory(none) uwtable
define noundef double @fixture_batch(i64 noundef %iterations, double noundef %seed) unnamed_addr #3 {
start:
  %_55.not.i = icmp eq i64 %iterations, 0
  br i1 %_55.not.i, label %_ZN12llvm_fixture10batch_with17hb5490f161783ce26E.exit, label %bb2.i

bb2.i:                                            ; preds = %start, %bb2.i
  %sum.sroa.0.07.i = phi double [ %7, %bb2.i ], [ 0.000000e+00, %start ]
  %i.sroa.0.06.i = phi i64 [ %8, %bb2.i ], [ 0, %start ]
  %0 = insertelement <2 x i64> poison, i64 %i.sroa.0.06.i, i64 0
  %1 = shufflevector <2 x i64> %0, <2 x i64> poison, <2 x i32> zeroinitializer
  %2 = and <2 x i64> %1, <i64 31, i64 7>
  %3 = uitofp nneg <2 x i64> %2 to <2 x double>
  %4 = fmul <2 x double> %3, <double 1.250000e-01, double 5.000000e-01>
  %5 = extractelement <2 x double> %4, i64 0
  %amount.i = fadd double %seed, %5
  %6 = extractelement <2 x double> %4, i64 1
  %_0.i.i.i.i.i.i.i = fsub double %amount.i, %6
  %_5.i.i.i.i.i = fcmp olt double %_0.i.i.i.i.i.i.i, 1.800000e+01
  %_3.sroa.0.0.i.i.i.i.i = select i1 %_5.i.i.i.i.i, double 1.800000e+01, double %_0.i.i.i.i.i.i.i
  %7 = fadd double %sum.sroa.0.07.i, %_3.sroa.0.0.i.i.i.i.i
  %8 = add nuw i64 %i.sroa.0.06.i, 1
  %exitcond.not.i = icmp eq i64 %8, %iterations
  br i1 %exitcond.not.i, label %_ZN12llvm_fixture10batch_with17hb5490f161783ce26E.exit, label %bb2.i

_ZN12llvm_fixture10batch_with17hb5490f161783ce26E.exit: ; preds = %bb2.i, %start
  %sum.sroa.0.0.lcssa.i = phi double [ 0.000000e+00, %start ], [ %7, %bb2.i ]
  ret double %sum.sroa.0.0.lcssa.i
}

; Function Attrs: mustprogress nocallback nofree nosync nounwind willreturn memory(argmem: readwrite)
declare void @llvm.lifetime.start.p0(i64 immarg, ptr captures(none)) #4

; Function Attrs: mustprogress nocallback nofree nosync nounwind willreturn memory(argmem: readwrite)
declare void @llvm.lifetime.end.p0(i64 immarg, ptr captures(none)) #4

; Function Attrs: mustprogress nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias writeonly captures(none), ptr noalias readonly captures(none), i64, i1 immarg) #5

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: readwrite) uwtable
define void @__rsmx_handler_16c5b70e7fc8c074fa7e8ca9(ptr noundef captures(none) %__address) unnamed_addr #2 {
start:
  %0 = load double, ptr %__address, align 8, !noundef !3
  %_5 = fcmp olt double %0, 1.800000e+01
  %_3.sroa.0.0 = select i1 %_5, double 1.800000e+01, double %0
  store double %_3.sroa.0.0, ptr %__address, align 8
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: readwrite) uwtable
define void @__rsmx_handler_type_16c5b70e7fc8c074fa7e8ca9(ptr noundef writeonly captures(none) initializes((0, 16)) %__output) unnamed_addr #2 {
start:
  tail call void @llvm.memcpy.p0.p0.i64(ptr noundef nonnull align 8 dereferenceable(16) %__output, ptr noundef nonnull align 8 dereferenceable(16) @anon.0639e836b9e1b73c458d36206545f83f.0, i64 16, i1 false)
  ret void
}

attributes #0 = { mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable "target-cpu"="x86-64" "target-features"="+cx16,+sse3,+sahf" }
attributes #1 = { noinline nounwind uwtable "target-cpu"="x86-64" "target-features"="+cx16,+sse3,+sahf" }
attributes #2 = { mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: readwrite) uwtable "target-cpu"="x86-64" "target-features"="+cx16,+sse3,+sahf" }
attributes #3 = { nofree norecurse nosync nounwind memory(none) uwtable "target-cpu"="x86-64" "target-features"="+cx16,+sse3,+sahf" }
attributes #4 = { mustprogress nocallback nofree nosync nounwind willreturn memory(argmem: readwrite) }
attributes #5 = { mustprogress nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #6 = { nounwind }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1, !1}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{!"rustc version 1.92.0 (ded5c06cf 2025-12-08)"}
!2 = !{i64 2893854475438031}
!3 = !{}
