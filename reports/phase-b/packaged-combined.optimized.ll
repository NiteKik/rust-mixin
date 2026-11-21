; ModuleID = 'retained-rust'
source_filename = "llvm_fixture.da7caf818defa757-cgu.0"
target datalayout = "e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-windows-msvc"

@__rsmx_meta_59aed09041305b4e1e46c395 = constant [200 x i8] c"{\22arity\22:2,\22entry\22:\22__rsmx_entry_08aa4a1cf4c3009dca5ed38f\22,\22kind\22:\22target\22,\22logical_id\22:\22demo::combat::apply_damage\22,\22retention_noinline\22:true,\22schema\22:1,\22symbol\22:\22__rsmx_fn_08aa4a1cf4c3009dca5ed38f\22}", align 1
@__rsmx_meta_794813aff581a360f1387746 = constant [200 x i8] c"{\22arity\22:2,\22entry\22:\22__rsmx_entry_6c5480d415ed2b55da7ba335\22,\22kind\22:\22target\22,\22logical_id\22:\22demo::combat::armor_reduce\22,\22retention_noinline\22:true,\22schema\22:1,\22symbol\22:\22__rsmx_fn_6c5480d415ed2b55da7ba335\22}", align 1
@__rsmx_meta_826f26bacc73c401cf7b9a6e = constant [196 x i8] c"{\22arity\22:2,\22entry\22:\22__rsmx_entry_3cb4fc20c82e364a35b1be9f\22,\22kind\22:\22target\22,\22logical_id\22:\22demo::combat::two_hits\22,\22retention_noinline\22:true,\22schema\22:1,\22symbol\22:\22__rsmx_fn_3cb4fc20c82e364a35b1be9f\22}", align 1
@__rsmx_meta_90db65816e2f8ab9db351e3a = constant [194 x i8] c"{\22arity\22:2,\22entry\22:\22__rsmx_entry_e02ec5709ce5f1a52a6d8b6f\22,\22kind\22:\22target\22,\22logical_id\22:\22demo::combat::caller\22,\22retention_noinline\22:true,\22schema\22:1,\22symbol\22:\22__rsmx_fn_e02ec5709ce5f1a52a6d8b6f\22}", align 1
@__rsmx_meta_dfe32c96c71be7c562c11a91 = constant [233 x i8] c"{\22callee\22:\22demo::combat::armor_reduce\22,\22id\22:\22adjust_damage\22,\22kind\22:\22modify_call_result\22,\22ordinal\22:0,\22priority\22:0,\22require_matches\22:1,\22schema\22:1,\22symbol\22:\22__rsmx_handler_6da65494c143e50f2b6515bf\22,\22target\22:\22demo::combat::apply_damage\22}", align 1
@__rsmx_meta_fdac69afdbbebb2ada723d1d = constant [231 x i8] c"{\22callee\22:\22demo::combat::armor_reduce\22,\22id\22:\22half_damage\22,\22kind\22:\22modify_call_result\22,\22ordinal\22:0,\22priority\22:0,\22require_matches\22:1,\22schema\22:1,\22symbol\22:\22__rsmx_handler_6f072205a065352a85adc84c\22,\22target\22:\22demo::combat::apply_damage\22}", align 1
@llvm.used = appending global [6 x ptr] [ptr @__rsmx_meta_59aed09041305b4e1e46c395, ptr @__rsmx_meta_794813aff581a360f1387746, ptr @__rsmx_meta_826f26bacc73c401cf7b9a6e, ptr @__rsmx_meta_90db65816e2f8ab9db351e3a, ptr @__rsmx_meta_dfe32c96c71be7c562c11a91, ptr @__rsmx_meta_fdac69afdbbebb2ada723d1d], section "llvm.metadata"

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @_ZN12llvm_fixture14fixture_damage17h58473050bfd61ec8E(double noundef %amount, double noundef %armor) unnamed_addr #0 {
start:
  %_3.i.i = fmul double %armor, 5.000000e-01
  %_0.i.i = fsub double %amount, %_3.i.i
  %_4.i.i = fcmp ogt double %_0.i.i, 1.000000e+01
  %..i.i = select i1 %_4.i.i, double 3.000000e+00, double 0.000000e+00
  %_3.i2.i = fadd double %_0.i.i, %..i.i
  %_0.i3.i = fmul double %_3.i2.i, 5.000000e-01
  %_0.i4.i = fmul double %_0.i3.i, 5.000000e-01
  ret double %_0.i4.i
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @_ZN12llvm_fixture20fixture_armor_reduce17hfb2b52ad1f9fc036E(double noundef %amount, double noundef %armor) unnamed_addr #0 {
start:
  %_3.i = fmul double %armor, 5.000000e-01
  %_0.i = fsub double %amount, %_3.i
  ret double %_0.i
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @__rsmx_entry_08aa4a1cf4c3009dca5ed38f(double noundef %__arg0, double noundef %__arg1) unnamed_addr #0 {
start:
  %_3.i.i = fmul double %__arg1, 5.000000e-01
  %_0.i.i = fsub double %__arg0, %_3.i.i
  %_4.i.i = fcmp ogt double %_0.i.i, 1.000000e+01
  %..i.i = select i1 %_4.i.i, double 3.000000e+00, double 0.000000e+00
  %_3.i2.i = fadd double %_0.i.i, %..i.i
  %_0.i3.i = fmul double %_3.i2.i, 5.000000e-01
  %_0.i4.i = fmul double %_0.i3.i, 5.000000e-01
  ret double %_0.i4.i
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @__rsmx_entry_3cb4fc20c82e364a35b1be9f(double noundef %__arg0, double noundef %__arg1) unnamed_addr #0 {
start:
  %_3.i.i = fmul double %__arg1, 5.000000e-01
  %_0.i.i = fsub double %__arg0, %_3.i.i
  %_5.i = fadd double %__arg0, 2.000000e+00
  %_0.i2.i = fsub double %_5.i, %_3.i.i
  %_0.i = fadd double %_0.i.i, %_0.i2.i
  ret double %_0.i
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @__rsmx_entry_6c5480d415ed2b55da7ba335(double noundef %__arg0, double noundef %__arg1) unnamed_addr #0 {
start:
  %_3.i = fmul double %__arg1, 5.000000e-01
  %_0.i = fsub double %__arg0, %_3.i
  ret double %_0.i
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @__rsmx_entry_e02ec5709ce5f1a52a6d8b6f(double noundef %__arg0, double noundef %__arg1) unnamed_addr #0 {
start:
  %_3.i.i.i = fmul double %__arg1, 5.000000e-01
  %_0.i.i.i = fsub double %__arg0, %_3.i.i.i
  %_4.i.i.i = fcmp ogt double %_0.i.i.i, 1.000000e+01
  %..i.i.i = select i1 %_4.i.i.i, double 3.000000e+00, double 0.000000e+00
  %_3.i2.i.i = fadd double %_0.i.i.i, %..i.i.i
  %_0.i3.i.i = fmul double %_3.i2.i.i, 5.000000e-01
  %_0.i4.i.i = fmul double %_0.i3.i.i, 5.000000e-01
  %_0.i = fadd double %_0.i4.i.i, 1.000000e+00
  ret double %_0.i
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @__rsmx_fn_08aa4a1cf4c3009dca5ed38f(double noundef %amount, double noundef %armor) unnamed_addr #0 {
start:
  %_3.i = fmul double %armor, 5.000000e-01
  %_0.i = fsub double %amount, %_3.i
  %_4.i = fcmp ogt double %_0.i, 1.000000e+01
  %..i = select i1 %_4.i, double 3.000000e+00, double 0.000000e+00
  %_3.i2 = fadd double %_0.i, %..i
  %_0.i3 = fmul double %_3.i2, 5.000000e-01
  %_0.i4 = fmul double %_0.i3, 5.000000e-01
  ret double %_0.i4
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @__rsmx_fn_3cb4fc20c82e364a35b1be9f(double noundef %amount, double noundef %armor) unnamed_addr #0 {
start:
  %_3.i = fmul double %armor, 5.000000e-01
  %_0.i = fsub double %amount, %_3.i
  %_5 = fadd double %amount, 2.000000e+00
  %_0.i2 = fsub double %_5, %_3.i
  %_0 = fadd double %_0.i, %_0.i2
  ret double %_0
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @__rsmx_fn_6c5480d415ed2b55da7ba335(double noundef %amount, double noundef %armor) unnamed_addr #0 {
start:
  %_3 = fmul double %armor, 5.000000e-01
  %_0 = fsub double %amount, %_3
  ret double %_0
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @__rsmx_fn_e02ec5709ce5f1a52a6d8b6f(double noundef %amount, double noundef %armor) unnamed_addr #0 {
start:
  %_3.i.i = fmul double %armor, 5.000000e-01
  %_0.i.i = fsub double %amount, %_3.i.i
  %_4.i.i = fcmp ogt double %_0.i.i, 1.000000e+01
  %..i.i = select i1 %_4.i.i, double 3.000000e+00, double 0.000000e+00
  %_3.i2.i = fadd double %_0.i.i, %..i.i
  %_0.i3.i = fmul double %_3.i2.i, 5.000000e-01
  %_0.i4.i = fmul double %_0.i3.i, 5.000000e-01
  %_0 = fadd double %_0.i4.i, 1.000000e+00
  ret double %_0
}

; Function Attrs: nofree norecurse nosync nounwind memory(none) uwtable
define noundef double @fixture_batch(i64 noundef %iterations, double noundef %seed) unnamed_addr #1 {
start:
  %_55.not.i = icmp eq i64 %iterations, 0
  br i1 %_55.not.i, label %_ZN12llvm_fixture10batch_with17h771d0c9bbaaab3a2E.exit, label %bb2.i

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
  %_0.i.i.i.i.i = fsub double %amount.i, %6
  %_4.i.i.i.i.i = fcmp ogt double %_0.i.i.i.i.i, 1.000000e+01
  %..i.i.i.i.i = select i1 %_4.i.i.i.i.i, double 3.000000e+00, double 0.000000e+00
  %_3.i2.i.i.i.i = fadd double %_0.i.i.i.i.i, %..i.i.i.i.i
  %_0.i3.i.i.i.i = fmul double %_3.i2.i.i.i.i, 5.000000e-01
  %_0.i4.i.i.i.i = fmul double %_0.i3.i.i.i.i, 5.000000e-01
  %7 = fadd double %sum.sroa.0.07.i, %_0.i4.i.i.i.i
  %8 = add nuw i64 %i.sroa.0.06.i, 1
  %exitcond.not.i = icmp eq i64 %8, %iterations
  br i1 %exitcond.not.i, label %_ZN12llvm_fixture10batch_with17h771d0c9bbaaab3a2E.exit, label %bb2.i

_ZN12llvm_fixture10batch_with17h771d0c9bbaaab3a2E.exit: ; preds = %bb2.i, %start
  %sum.sroa.0.0.lcssa.i = phi double [ 0.000000e+00, %start ], [ %7, %bb2.i ]
  ret double %sum.sroa.0.0.lcssa.i
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @__rsmx_handler_6da65494c143e50f2b6515bf(double noundef %value) unnamed_addr #0 {
start:
  %_4 = fcmp ogt double %value, 1.000000e+01
  %. = select i1 %_4, double 3.000000e+00, double 0.000000e+00
  %_3 = fadd double %value, %.
  %_0 = fmul double %_3, 5.000000e-01
  ret double %_0
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @__rsmx_handler_6f072205a065352a85adc84c(double noundef %value) unnamed_addr #0 {
start:
  %_0 = fmul double %value, 5.000000e-01
  ret double %_0
}

attributes #0 = { mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable "target-cpu"="x86-64" "target-features"="+cx16,+sse3,+sahf" }
attributes #1 = { nofree norecurse nosync nounwind memory(none) uwtable "target-cpu"="x86-64" "target-features"="+cx16,+sse3,+sahf" }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1, !1, !1}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{!"rustc version 1.92.0 (ded5c06cf 2025-12-08)"}
