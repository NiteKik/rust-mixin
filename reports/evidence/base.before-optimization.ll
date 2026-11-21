; ModuleID = 'retained-rust'
source_filename = "retained_game.8540c94749587d2c-cgu.0"
target datalayout = "e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-windows-msvc"

; Function Attrs: inlinehint nounwind uwtable
define internal noundef double @_ZN13retained_game10batch_with17h6ed0ad826b3392fcE(i64 noundef %iterations, double noundef %seed) unnamed_addr #0 {
start:
  %i = alloca [8 x i8], align 8
  %sum = alloca [8 x i8], align 8
  %f = alloca [0 x i8], align 1
  store double 0.000000e+00, ptr %sum, align 8
  call void @llvm.lifetime.start.p0(i64 8, ptr %i)
  store i64 0, ptr %i, align 8
  br label %bb1

bb1:                                              ; preds = %bb2, %start
  %_6 = load i64, ptr %i, align 8, !noundef !2
  %_5 = icmp ult i64 %_6, %iterations
  br i1 %_5, label %bb2, label %bb4

bb4:                                              ; preds = %bb1
  call void @llvm.lifetime.end.p0(i64 8, ptr %i)
  %0 = load double, ptr %sum, align 8, !noundef !2
  ret double %0

bb2:                                              ; preds = %bb1
  %_11 = load i64, ptr %i, align 8, !noundef !2
  %_10 = and i64 %_11, 31
  %_9 = uitofp i64 %_10 to double
  %_8 = fmul double %_9, 1.250000e-01
  %amount = fadd double %seed, %_8
  %_14 = load i64, ptr %i, align 8, !noundef !2
  %_13 = and i64 %_14, 7
  %armor = uitofp i64 %_13 to double
  %_15 = call noundef double @"_ZN13retained_game13fixture_batch28_$u7b$$u7b$closure$u7d$$u7d$17h72eb59377f38105aE"(ptr noalias noundef nonnull align 1 %f, double noundef %amount, double noundef %armor) #3
  %1 = load double, ptr %sum, align 8, !noundef !2
  %2 = fadd double %1, %_15
  store double %2, ptr %sum, align 8
  %3 = load i64, ptr %i, align 8, !noundef !2
  %4 = add i64 %3, 1
  store i64 %4, ptr %i, align 8
  br label %bb1
}

; Function Attrs: inlinehint nounwind uwtable
define internal noundef double @"_ZN13retained_game13fixture_batch28_$u7b$$u7b$closure$u7d$$u7d$17h72eb59377f38105aE"(ptr noalias noundef nonnull align 1 %_1, double noundef %amount, double noundef %armor) unnamed_addr #0 {
start:
  %_0 = call noundef double @fixture_damage(double noundef %amount, double noundef %armor) #3
  ret double %_0
}

; Function Attrs: nounwind uwtable
define noundef double @fixture_armor_reduce(double noundef %amount, double noundef %armor) unnamed_addr #1 {
start:
  %_3 = fmul double %armor, 5.000000e-01
  %_0 = fsub double %amount, %_3
  ret double %_0
}

; Function Attrs: nounwind uwtable
define noundef double @fixture_batch(i64 noundef %iterations, double noundef %seed) unnamed_addr #1 {
start:
  %_0 = call noundef double @_ZN13retained_game10batch_with17h6ed0ad826b3392fcE(i64 noundef %iterations, double noundef %seed) #3
  ret double %_0
}

; Function Attrs: nounwind uwtable
define noundef double @fixture_damage(double noundef %amount, double noundef %armor) unnamed_addr #1 {
start:
  %_0 = call noundef double @fixture_armor_reduce(double noundef %amount, double noundef %armor) #3
  ret double %_0
}

; Function Attrs: nocallback nofree nosync nounwind willreturn memory(argmem: readwrite)
declare void @llvm.lifetime.start.p0(i64 immarg, ptr captures(none)) #2

; Function Attrs: nocallback nofree nosync nounwind willreturn memory(argmem: readwrite)
declare void @llvm.lifetime.end.p0(i64 immarg, ptr captures(none)) #2

; Function Attrs: nounwind uwtable
define noundef double @mod_adjust_damage(double noundef %value) unnamed_addr #1 {
start:
  %bonus = alloca [8 x i8], align 8
  %_4 = fcmp ogt double %value, 1.000000e+01
  br i1 %_4, label %bb1, label %bb2

bb2:                                              ; preds = %start
  store double 0.000000e+00, ptr %bonus, align 8
  br label %bb3

bb1:                                              ; preds = %start
  store double 3.000000e+00, ptr %bonus, align 8
  br label %bb3

bb3:                                              ; preds = %bb1, %bb2
  %0 = load double, ptr %bonus, align 8, !noundef !2
  %_3 = fadd double %value, %0
  %_0 = fmul double %_3, 5.000000e-01
  ret double %_0
}

attributes #0 = { inlinehint nounwind uwtable "target-cpu"="x86-64" "target-features"="+cx16,+sse3,+sahf" }
attributes #1 = { nounwind uwtable "target-cpu"="x86-64" "target-features"="+cx16,+sse3,+sahf" }
attributes #2 = { nocallback nofree nosync nounwind willreturn memory(argmem: readwrite) }
attributes #3 = { nounwind }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1, !1}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{!"rustc version 1.92.0 (ded5c06cf 2025-12-08)"}
!2 = !{}
