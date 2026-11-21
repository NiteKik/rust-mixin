; ModuleID = 'retained-rust'
source_filename = "retained_game.8540c94749587d2c-cgu.0"
target datalayout = "e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-windows-msvc"

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @fixture_armor_reduce(double noundef %amount, double noundef %armor) unnamed_addr #0 {
start:
  %_3 = fmul double %armor, 5.000000e-01
  %_0 = fsub double %amount, %_3
  ret double %_0
}

; Function Attrs: nofree norecurse nosync nounwind memory(none) uwtable
define noundef double @fixture_batch(i64 noundef %iterations, double noundef %seed) unnamed_addr #1 {
start:
  %_55.not.i = icmp eq i64 %iterations, 0
  br i1 %_55.not.i, label %_ZN13retained_game10batch_with17h6ed0ad826b3392fcE.exit, label %bb2.i

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
  %_0.i.i.i.i = fsub double %amount.i, %6
  %_4.i.i.i.i = fcmp ogt double %_0.i.i.i.i, 1.000000e+01
  %..i.i.i.i = select i1 %_4.i.i.i.i, double 3.000000e+00, double 0.000000e+00
  %_3.i1.i.i.i = fadd double %_0.i.i.i.i, %..i.i.i.i
  %_0.i2.i.i.i = fmul double %_3.i1.i.i.i, 5.000000e-01
  %7 = fadd double %sum.sroa.0.07.i, %_0.i2.i.i.i
  %8 = add nuw i64 %i.sroa.0.06.i, 1
  %exitcond.not.i = icmp eq i64 %8, %iterations
  br i1 %exitcond.not.i, label %_ZN13retained_game10batch_with17h6ed0ad826b3392fcE.exit, label %bb2.i

_ZN13retained_game10batch_with17h6ed0ad826b3392fcE.exit: ; preds = %bb2.i, %start
  %sum.sroa.0.0.lcssa.i = phi double [ 0.000000e+00, %start ], [ %7, %bb2.i ]
  ret double %sum.sroa.0.0.lcssa.i
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @fixture_damage(double noundef %amount, double noundef %armor) unnamed_addr #0 {
start:
  %_3.i = fmul double %armor, 5.000000e-01
  %_0.i = fsub double %amount, %_3.i
  %_4.i = fcmp ogt double %_0.i, 1.000000e+01
  %..i = select i1 %_4.i, double 3.000000e+00, double 0.000000e+00
  %_3.i1 = fadd double %_0.i, %..i
  %_0.i2 = fmul double %_3.i1, 5.000000e-01
  ret double %_0.i2
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @mod_adjust_damage(double noundef %value) unnamed_addr #0 {
start:
  %_4 = fcmp ogt double %value, 1.000000e+01
  %. = select i1 %_4, double 3.000000e+00, double 0.000000e+00
  %_3 = fadd double %value, %.
  %_0 = fmul double %_3, 5.000000e-01
  ret double %_0
}

attributes #0 = { mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable "target-cpu"="x86-64" "target-features"="+cx16,+sse3,+sahf" }
attributes #1 = { nofree norecurse nosync nounwind memory(none) uwtable "target-cpu"="x86-64" "target-features"="+cx16,+sse3,+sahf" }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1, !1}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{!"rustc version 1.92.0 (ded5c06cf 2025-12-08)"}
