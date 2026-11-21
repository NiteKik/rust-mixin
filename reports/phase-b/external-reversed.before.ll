; ModuleID = 'retained-rust'
source_filename = "llvm_fixture.45319190217dc304-cgu.0"
target datalayout = "e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-windows-msvc"

@__rsmx_meta_59aed09041305b4e1e46c395 = constant [371 x i8] c"{\22arity\22:2,\22entry\22:\22__rsmx_entry_08aa4a1cf4c3009dca5ed38f\22,\22kind\22:\22target\22,\22logical_id\22:\22demo::combat::apply_damage\22,\22retention_noinline\22:true,\22return_hook\22:{\22body\22:\22__rsmx_body_08aa4a1cf4c3009dca5ed38f\22,\22symbol\22:\22__rsmx_return_08aa4a1cf4c3009dca5ed38f\22,\22type_probe\22:\22__rsmx_target_type_08aa4a1cf4c3009dca5ed38f\22},\22schema\22:1,\22symbol\22:\22__rsmx_fn_08aa4a1cf4c3009dca5ed38f\22}", align 1
@__rsmx_meta_794813aff581a360f1387746 = constant [371 x i8] c"{\22arity\22:2,\22entry\22:\22__rsmx_entry_6c5480d415ed2b55da7ba335\22,\22kind\22:\22target\22,\22logical_id\22:\22demo::combat::armor_reduce\22,\22retention_noinline\22:true,\22return_hook\22:{\22body\22:\22__rsmx_body_6c5480d415ed2b55da7ba335\22,\22symbol\22:\22__rsmx_return_6c5480d415ed2b55da7ba335\22,\22type_probe\22:\22__rsmx_target_type_6c5480d415ed2b55da7ba335\22},\22schema\22:1,\22symbol\22:\22__rsmx_fn_6c5480d415ed2b55da7ba335\22}", align 1
@__rsmx_meta_826f26bacc73c401cf7b9a6e = constant [367 x i8] c"{\22arity\22:2,\22entry\22:\22__rsmx_entry_3cb4fc20c82e364a35b1be9f\22,\22kind\22:\22target\22,\22logical_id\22:\22demo::combat::two_hits\22,\22retention_noinline\22:true,\22return_hook\22:{\22body\22:\22__rsmx_body_3cb4fc20c82e364a35b1be9f\22,\22symbol\22:\22__rsmx_return_3cb4fc20c82e364a35b1be9f\22,\22type_probe\22:\22__rsmx_target_type_3cb4fc20c82e364a35b1be9f\22},\22schema\22:1,\22symbol\22:\22__rsmx_fn_3cb4fc20c82e364a35b1be9f\22}", align 1
@__rsmx_meta_90db65816e2f8ab9db351e3a = constant [365 x i8] c"{\22arity\22:2,\22entry\22:\22__rsmx_entry_e02ec5709ce5f1a52a6d8b6f\22,\22kind\22:\22target\22,\22logical_id\22:\22demo::combat::caller\22,\22retention_noinline\22:true,\22return_hook\22:{\22body\22:\22__rsmx_body_e02ec5709ce5f1a52a6d8b6f\22,\22symbol\22:\22__rsmx_return_e02ec5709ce5f1a52a6d8b6f\22,\22type_probe\22:\22__rsmx_target_type_e02ec5709ce5f1a52a6d8b6f\22},\22schema\22:1,\22symbol\22:\22__rsmx_fn_e02ec5709ce5f1a52a6d8b6f\22}", align 1
@anon.ccdecddb22efeedd8ddd9ba6d8e00dcf.0 = private unnamed_addr constant <{ ptr, ptr }> <{ ptr inttoptr (i64 794668071241239420 to ptr), ptr inttoptr (i64 -4297518438299490226 to ptr) }>, align 8
@llvm.used = appending global [6 x ptr] [ptr @__rsmx_meta_59aed09041305b4e1e46c395, ptr @__rsmx_meta_794813aff581a360f1387746, ptr @__rsmx_meta_826f26bacc73c401cf7b9a6e, ptr @__rsmx_meta_90db65816e2f8ab9db351e3a, ptr @__rsmx_meta_fdac69afdbbebb2ada723d1d, ptr @__rsmx_meta_dfe32c96c71be7c562c11a91], section "llvm.metadata"
@__rsmx_meta_fdac69afdbbebb2ada723d1d = constant [281 x i8] c"{\22callee\22:\22demo::combat::armor_reduce\22,\22id\22:\22half_damage\22,\22kind\22:\22modify_call_result\22,\22operation\22:\22modify_call_result\22,\22ordinal\22:0,\22parameter\22:null,\22priority\22:0,\22require_matches\22:1,\22schema\22:1,\22symbol\22:\22__rsmx_handler_6f072205a065352a85adc84c\22,\22target\22:\22demo::combat::apply_damage\22}", align 1
@__rsmx_meta_dfe32c96c71be7c562c11a91 = constant [283 x i8] c"{\22callee\22:\22demo::combat::armor_reduce\22,\22id\22:\22adjust_damage\22,\22kind\22:\22modify_call_result\22,\22operation\22:\22modify_call_result\22,\22ordinal\22:0,\22parameter\22:null,\22priority\22:0,\22require_matches\22:1,\22schema\22:1,\22symbol\22:\22__rsmx_handler_6da65494c143e50f2b6515bf\22,\22target\22:\22demo::combat::apply_damage\22}", align 1

; Function Attrs: inlinehint nounwind uwtable
define internal noundef double @_ZN12llvm_fixture10batch_with17hb5490f161783ce26E(i64 noundef %iterations, double noundef %seed) unnamed_addr #0 {
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
  %_15 = call noundef double @"_ZN12llvm_fixture13fixture_batch28_$u7b$$u7b$closure$u7d$$u7d$17he84da5e46cb20197E"(ptr noalias noundef nonnull align 1 %f, double noundef %amount, double noundef %armor) #5
  %1 = load double, ptr %sum, align 8, !noundef !2
  %2 = fadd double %1, %_15
  store double %2, ptr %sum, align 8
  %3 = load i64, ptr %i, align 8, !noundef !2
  %4 = add i64 %3, 1
  store i64 %4, ptr %i, align 8
  br label %bb1
}

; Function Attrs: inlinehint nounwind uwtable
define internal noundef double @"_ZN12llvm_fixture13fixture_batch28_$u7b$$u7b$closure$u7d$$u7d$17he84da5e46cb20197E"(ptr noalias noundef nonnull align 1 %_1, double noundef %amount, double noundef %armor) unnamed_addr #0 {
start:
  %_0 = call noundef double @_ZN12llvm_fixture14fixture_damage17hef4468a5d7332544E(double noundef %amount, double noundef %armor) #5
  ret double %_0
}

; Function Attrs: nounwind uwtable
define noundef double @_ZN12llvm_fixture14fixture_damage17hef4468a5d7332544E(double noundef %amount, double noundef %armor) unnamed_addr #1 {
start:
  %_0 = call noundef double @__rsmx_fn_08aa4a1cf4c3009dca5ed38f(double noundef %amount, double noundef %armor) #5
  ret double %_0
}

; Function Attrs: nounwind uwtable
define noundef double @_ZN12llvm_fixture20fixture_armor_reduce17hafa8a93c603f5ad8E(double noundef %amount, double noundef %armor) unnamed_addr #1 {
start:
  %_0 = call noundef double @__rsmx_fn_6c5480d415ed2b55da7ba335(double noundef %amount, double noundef %armor) #5
  ret double %_0
}

; Function Attrs: nounwind uwtable
define noundef double @__rsmx_body_08aa4a1cf4c3009dca5ed38f(double noundef %amount, double noundef %armor) unnamed_addr #1 {
start:
  %_0 = call noundef double @__rsmx_fn_6c5480d415ed2b55da7ba335(double noundef %amount, double noundef %armor) #5
  %mixin.result = call double @__rsmx_handler_6f072205a065352a85adc84c(double %_0)
  %mixin.result1 = call double @__rsmx_handler_6da65494c143e50f2b6515bf(double %mixin.result)
  ret double %mixin.result1
}

; Function Attrs: nounwind uwtable
define noundef double @__rsmx_body_3cb4fc20c82e364a35b1be9f(double noundef %amount, double noundef %armor) unnamed_addr #1 {
start:
  %_3 = call noundef double @__rsmx_fn_6c5480d415ed2b55da7ba335(double noundef %amount, double noundef %armor) #5
  %_5 = fadd double %amount, 2.000000e+00
  %_4 = call noundef double @__rsmx_fn_6c5480d415ed2b55da7ba335(double noundef %_5, double noundef %armor) #5
  %_0 = fadd double %_3, %_4
  ret double %_0
}

; Function Attrs: nounwind uwtable
define noundef double @__rsmx_body_6c5480d415ed2b55da7ba335(double noundef %amount, double noundef %armor) unnamed_addr #1 {
start:
  %_3 = fmul double %armor, 5.000000e-01
  %_0 = fsub double %amount, %_3
  ret double %_0
}

; Function Attrs: nounwind uwtable
define noundef double @__rsmx_body_e02ec5709ce5f1a52a6d8b6f(double noundef %amount, double noundef %armor) unnamed_addr #1 {
start:
  %_3 = call noundef double @__rsmx_fn_08aa4a1cf4c3009dca5ed38f(double noundef %amount, double noundef %armor) #5
  %_0 = fadd double %_3, 1.000000e+00
  ret double %_0
}

; Function Attrs: nounwind uwtable
define noundef double @__rsmx_entry_08aa4a1cf4c3009dca5ed38f(double noundef %__arg0, double noundef %__arg1) unnamed_addr #1 {
start:
  %_0 = call noundef double @__rsmx_fn_08aa4a1cf4c3009dca5ed38f(double noundef %__arg0, double noundef %__arg1) #5
  ret double %_0
}

; Function Attrs: nounwind uwtable
define noundef double @__rsmx_entry_3cb4fc20c82e364a35b1be9f(double noundef %__arg0, double noundef %__arg1) unnamed_addr #1 {
start:
  %_0 = call noundef double @__rsmx_fn_3cb4fc20c82e364a35b1be9f(double noundef %__arg0, double noundef %__arg1) #5
  ret double %_0
}

; Function Attrs: nounwind uwtable
define noundef double @__rsmx_entry_6c5480d415ed2b55da7ba335(double noundef %__arg0, double noundef %__arg1) unnamed_addr #1 {
start:
  %_0 = call noundef double @__rsmx_fn_6c5480d415ed2b55da7ba335(double noundef %__arg0, double noundef %__arg1) #5
  ret double %_0
}

; Function Attrs: nounwind uwtable
define noundef double @__rsmx_entry_e02ec5709ce5f1a52a6d8b6f(double noundef %__arg0, double noundef %__arg1) unnamed_addr #1 {
start:
  %_0 = call noundef double @__rsmx_fn_e02ec5709ce5f1a52a6d8b6f(double noundef %__arg0, double noundef %__arg1) #5
  ret double %_0
}

; Function Attrs: nounwind uwtable
define noundef double @__rsmx_fn_08aa4a1cf4c3009dca5ed38f(double noundef %__mixin_input0, double noundef %__mixin_input1) unnamed_addr #1 {
start:
  %__result = alloca [8 x i8], align 8
  call void @llvm.lifetime.start.p0(i64 8, ptr %__result)
  %0 = call noundef double @__rsmx_body_08aa4a1cf4c3009dca5ed38f(double noundef %__mixin_input0, double noundef %__mixin_input1) #5
  store double %0, ptr %__result, align 8
  %_0 = load double, ptr %__result, align 8, !noundef !2
  call void @llvm.lifetime.end.p0(i64 8, ptr %__result)
  ret double %_0
}

; Function Attrs: nounwind uwtable
define noundef double @__rsmx_fn_3cb4fc20c82e364a35b1be9f(double noundef %__mixin_input0, double noundef %__mixin_input1) unnamed_addr #1 {
start:
  %__result = alloca [8 x i8], align 8
  call void @llvm.lifetime.start.p0(i64 8, ptr %__result)
  %0 = call noundef double @__rsmx_body_3cb4fc20c82e364a35b1be9f(double noundef %__mixin_input0, double noundef %__mixin_input1) #5
  store double %0, ptr %__result, align 8
  %_0 = load double, ptr %__result, align 8, !noundef !2
  call void @llvm.lifetime.end.p0(i64 8, ptr %__result)
  ret double %_0
}

; Function Attrs: nounwind uwtable
define noundef double @__rsmx_fn_6c5480d415ed2b55da7ba335(double noundef %__mixin_input0, double noundef %__mixin_input1) unnamed_addr #1 {
start:
  %__result = alloca [8 x i8], align 8
  call void @llvm.lifetime.start.p0(i64 8, ptr %__result)
  %0 = call noundef double @__rsmx_body_6c5480d415ed2b55da7ba335(double noundef %__mixin_input0, double noundef %__mixin_input1) #5
  store double %0, ptr %__result, align 8
  %_0 = load double, ptr %__result, align 8, !noundef !2
  call void @llvm.lifetime.end.p0(i64 8, ptr %__result)
  ret double %_0
}

; Function Attrs: nounwind uwtable
define noundef double @__rsmx_fn_e02ec5709ce5f1a52a6d8b6f(double noundef %__mixin_input0, double noundef %__mixin_input1) unnamed_addr #1 {
start:
  %__result = alloca [8 x i8], align 8
  call void @llvm.lifetime.start.p0(i64 8, ptr %__result)
  %0 = call noundef double @__rsmx_body_e02ec5709ce5f1a52a6d8b6f(double noundef %__mixin_input0, double noundef %__mixin_input1) #5
  store double %0, ptr %__result, align 8
  %_0 = load double, ptr %__result, align 8, !noundef !2
  call void @llvm.lifetime.end.p0(i64 8, ptr %__result)
  ret double %_0
}

; Function Attrs: noinline nounwind uwtable
define void @__rsmx_return_08aa4a1cf4c3009dca5ed38f(ptr noundef %result) unnamed_addr #2 {
start:
  %0 = alloca [8 x i8], align 8
  call void @llvm.lifetime.start.p0(i64 8, ptr %0)
  store ptr %result, ptr %0, align 8
  call void asm sideeffect "", "r,~{memory}"(ptr %0), !srcloc !3
  %_2 = load ptr, ptr %0, align 8, !noundef !2
  call void @llvm.lifetime.end.p0(i64 8, ptr %0)
  ret void
}

; Function Attrs: noinline nounwind uwtable
define void @__rsmx_return_3cb4fc20c82e364a35b1be9f(ptr noundef %result) unnamed_addr #2 {
start:
  %0 = alloca [8 x i8], align 8
  call void @llvm.lifetime.start.p0(i64 8, ptr %0)
  store ptr %result, ptr %0, align 8
  call void asm sideeffect "", "r,~{memory}"(ptr %0), !srcloc !3
  %_2 = load ptr, ptr %0, align 8, !noundef !2
  call void @llvm.lifetime.end.p0(i64 8, ptr %0)
  ret void
}

; Function Attrs: noinline nounwind uwtable
define void @__rsmx_return_6c5480d415ed2b55da7ba335(ptr noundef %result) unnamed_addr #2 {
start:
  %0 = alloca [8 x i8], align 8
  call void @llvm.lifetime.start.p0(i64 8, ptr %0)
  store ptr %result, ptr %0, align 8
  call void asm sideeffect "", "r,~{memory}"(ptr %0), !srcloc !3
  %_2 = load ptr, ptr %0, align 8, !noundef !2
  call void @llvm.lifetime.end.p0(i64 8, ptr %0)
  ret void
}

; Function Attrs: noinline nounwind uwtable
define void @__rsmx_return_e02ec5709ce5f1a52a6d8b6f(ptr noundef %result) unnamed_addr #2 {
start:
  %0 = alloca [8 x i8], align 8
  call void @llvm.lifetime.start.p0(i64 8, ptr %0)
  store ptr %result, ptr %0, align 8
  call void asm sideeffect "", "r,~{memory}"(ptr %0), !srcloc !3
  %_2 = load ptr, ptr %0, align 8, !noundef !2
  call void @llvm.lifetime.end.p0(i64 8, ptr %0)
  ret void
}

; Function Attrs: nounwind uwtable
define void @__rsmx_target_type_08aa4a1cf4c3009dca5ed38f(ptr noundef %__output) unnamed_addr #1 {
start:
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %__output, ptr align 8 @anon.ccdecddb22efeedd8ddd9ba6d8e00dcf.0, i64 16, i1 false)
  ret void
}

; Function Attrs: nounwind uwtable
define void @__rsmx_target_type_3cb4fc20c82e364a35b1be9f(ptr noundef %__output) unnamed_addr #1 {
start:
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %__output, ptr align 8 @anon.ccdecddb22efeedd8ddd9ba6d8e00dcf.0, i64 16, i1 false)
  ret void
}

; Function Attrs: nounwind uwtable
define void @__rsmx_target_type_6c5480d415ed2b55da7ba335(ptr noundef %__output) unnamed_addr #1 {
start:
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %__output, ptr align 8 @anon.ccdecddb22efeedd8ddd9ba6d8e00dcf.0, i64 16, i1 false)
  ret void
}

; Function Attrs: nounwind uwtable
define void @__rsmx_target_type_e02ec5709ce5f1a52a6d8b6f(ptr noundef %__output) unnamed_addr #1 {
start:
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %__output, ptr align 8 @anon.ccdecddb22efeedd8ddd9ba6d8e00dcf.0, i64 16, i1 false)
  ret void
}

; Function Attrs: nounwind uwtable
define noundef double @fixture_batch(i64 noundef %iterations, double noundef %seed) unnamed_addr #1 {
start:
  %_0 = call noundef double @_ZN12llvm_fixture10batch_with17hb5490f161783ce26E(i64 noundef %iterations, double noundef %seed) #5
  ret double %_0
}

; Function Attrs: nocallback nofree nosync nounwind willreturn memory(argmem: readwrite)
declare void @llvm.lifetime.start.p0(i64 immarg, ptr captures(none)) #3

; Function Attrs: nocallback nofree nosync nounwind willreturn memory(argmem: readwrite)
declare void @llvm.lifetime.end.p0(i64 immarg, ptr captures(none)) #3

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias writeonly captures(none), ptr noalias readonly captures(none), i64, i1 immarg) #4

; Function Attrs: nounwind uwtable
define noundef double @__rsmx_handler_6f072205a065352a85adc84c(double noundef %value) unnamed_addr #1 {
start:
  %_0 = fmul double %value, 5.000000e-01
  ret double %_0
}

; Function Attrs: nounwind uwtable
define noundef double @__rsmx_handler_6da65494c143e50f2b6515bf(double noundef %value) unnamed_addr #1 {
start:
  %_2 = alloca [8 x i8], align 8
  call void @llvm.lifetime.start.p0(i64 8, ptr %_2)
  %_4 = fcmp ogt double %value, 1.000000e+01
  br i1 %_4, label %bb1, label %bb2

bb2:                                              ; preds = %start
  store double 0.000000e+00, ptr %_2, align 8
  br label %bb3

bb1:                                              ; preds = %start
  store double 3.000000e+00, ptr %_2, align 8
  br label %bb3

bb3:                                              ; preds = %bb1, %bb2
  %0 = load double, ptr %_2, align 8, !noundef !2
  %_3 = fadd double %value, %0
  %_0 = fmul double %_3, 5.000000e-01
  call void @llvm.lifetime.end.p0(i64 8, ptr %_2)
  ret double %_0
}

attributes #0 = { inlinehint nounwind uwtable "target-cpu"="x86-64" "target-features"="+cx16,+sse3,+sahf" }
attributes #1 = { nounwind uwtable "target-cpu"="x86-64" "target-features"="+cx16,+sse3,+sahf" }
attributes #2 = { noinline nounwind uwtable "target-cpu"="x86-64" "target-features"="+cx16,+sse3,+sahf" }
attributes #3 = { nocallback nofree nosync nounwind willreturn memory(argmem: readwrite) }
attributes #4 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #5 = { nounwind }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1, !1, !1}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{!"rustc version 1.92.0 (ded5c06cf 2025-12-08)"}
!2 = !{}
!3 = !{i64 2893854475438031}
