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
  br i1 %_55.not.i, label %_ZN13retained_game10batch_with17h6ed0ad826b3392fcE.exit, label %bb2.i.preheader

bb2.i.preheader:                                  ; preds = %start
  %xtraiter = and i64 %iterations, 1
  %0 = icmp eq i64 %iterations, 1
  br i1 %0, label %_ZN13retained_game10batch_with17h6ed0ad826b3392fcE.exit.loopexit.unr-lcssa, label %bb2.i.preheader.new

bb2.i.preheader.new:                              ; preds = %bb2.i.preheader
  %unroll_iter = and i64 %iterations, -2
  br label %bb2.i

bb2.i:                                            ; preds = %bb2.i, %bb2.i.preheader.new
  %sum.sroa.0.07.i = phi double [ 0.000000e+00, %bb2.i.preheader.new ], [ %17, %bb2.i ]
  %i.sroa.0.06.i = phi i64 [ 0, %bb2.i.preheader.new ], [ %18, %bb2.i ]
  %niter = phi i64 [ 0, %bb2.i.preheader.new ], [ %niter.next.1, %bb2.i ]
  %1 = insertelement <2 x i64> poison, i64 %i.sroa.0.06.i, i64 0
  %2 = shufflevector <2 x i64> %1, <2 x i64> poison, <2 x i32> zeroinitializer
  %3 = and <2 x i64> %2, <i64 31, i64 7>
  %4 = uitofp nneg <2 x i64> %3 to <2 x double>
  %5 = fmul <2 x double> %4, <double 1.250000e-01, double 5.000000e-01>
  %6 = extractelement <2 x double> %5, i64 0
  %amount.i = fadd double %seed, %6
  %7 = extractelement <2 x double> %5, i64 1
  %_0.i.i.i.i = fsub double %amount.i, %7
  %8 = fadd double %sum.sroa.0.07.i, %_0.i.i.i.i
  %9 = or disjoint i64 %i.sroa.0.06.i, 1
  %10 = insertelement <2 x i64> poison, i64 %9, i64 0
  %11 = shufflevector <2 x i64> %10, <2 x i64> poison, <2 x i32> zeroinitializer
  %12 = and <2 x i64> %11, <i64 31, i64 7>
  %13 = uitofp nneg <2 x i64> %12 to <2 x double>
  %14 = fmul <2 x double> %13, <double 1.250000e-01, double 5.000000e-01>
  %15 = extractelement <2 x double> %14, i64 0
  %amount.i.1 = fadd double %seed, %15
  %16 = extractelement <2 x double> %14, i64 1
  %_0.i.i.i.i.1 = fsub double %amount.i.1, %16
  %17 = fadd double %8, %_0.i.i.i.i.1
  %18 = add nuw i64 %i.sroa.0.06.i, 2
  %niter.next.1 = add i64 %niter, 2
  %niter.ncmp.1 = icmp eq i64 %niter.next.1, %unroll_iter
  br i1 %niter.ncmp.1, label %_ZN13retained_game10batch_with17h6ed0ad826b3392fcE.exit.loopexit.unr-lcssa, label %bb2.i

_ZN13retained_game10batch_with17h6ed0ad826b3392fcE.exit.loopexit.unr-lcssa: ; preds = %bb2.i, %bb2.i.preheader
  %.lcssa.ph = phi double [ poison, %bb2.i.preheader ], [ %17, %bb2.i ]
  %sum.sroa.0.07.i.unr = phi double [ 0.000000e+00, %bb2.i.preheader ], [ %17, %bb2.i ]
  %i.sroa.0.06.i.unr = phi i64 [ 0, %bb2.i.preheader ], [ %18, %bb2.i ]
  %lcmp.mod.not = icmp eq i64 %xtraiter, 0
  br i1 %lcmp.mod.not, label %_ZN13retained_game10batch_with17h6ed0ad826b3392fcE.exit, label %bb2.i.epil

bb2.i.epil:                                       ; preds = %_ZN13retained_game10batch_with17h6ed0ad826b3392fcE.exit.loopexit.unr-lcssa
  %19 = insertelement <2 x i64> poison, i64 %i.sroa.0.06.i.unr, i64 0
  %20 = shufflevector <2 x i64> %19, <2 x i64> poison, <2 x i32> zeroinitializer
  %21 = and <2 x i64> %20, <i64 31, i64 7>
  %22 = uitofp nneg <2 x i64> %21 to <2 x double>
  %23 = fmul <2 x double> %22, <double 1.250000e-01, double 5.000000e-01>
  %24 = extractelement <2 x double> %23, i64 0
  %amount.i.epil = fadd double %seed, %24
  %25 = extractelement <2 x double> %23, i64 1
  %_0.i.i.i.i.epil = fsub double %amount.i.epil, %25
  %26 = fadd double %sum.sroa.0.07.i.unr, %_0.i.i.i.i.epil
  br label %_ZN13retained_game10batch_with17h6ed0ad826b3392fcE.exit

_ZN13retained_game10batch_with17h6ed0ad826b3392fcE.exit: ; preds = %bb2.i.epil, %_ZN13retained_game10batch_with17h6ed0ad826b3392fcE.exit.loopexit.unr-lcssa, %start
  %sum.sroa.0.0.lcssa.i = phi double [ 0.000000e+00, %start ], [ %.lcssa.ph, %_ZN13retained_game10batch_with17h6ed0ad826b3392fcE.exit.loopexit.unr-lcssa ], [ %26, %bb2.i.epil ]
  ret double %sum.sroa.0.0.lcssa.i
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none) uwtable
define noundef double @fixture_damage(double noundef %amount, double noundef %armor) unnamed_addr #0 {
start:
  %_3.i = fmul double %armor, 5.000000e-01
  %_0.i = fsub double %amount, %_3.i
  ret double %_0.i
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
