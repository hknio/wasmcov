; ModuleID = 'tests/fibonacci.c'
source_filename = "tests/fibonacci.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

$__covrec_DB956436E78DD5FAu = comdat any

$__profc_main = comdat nodeduplicate

@.str = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@__covrec_DB956436E78DD5FAu = linkonce_odr hidden constant <{ i64, i32, i64, i64, [98 x i8] }> <{ i64 -2624081020897602054, i32 98, i64 1245365802964056, i64 -3989333466014016708, [98 x i8] c"\01\01\04\01\05\05\09\05\09\05\09\0E\01\04\01\14\02\10\05\01\00\01\03\01\14\00\1D \05\01\00\14\00\1D\05\00\1F\00\22\05\00$\01\84\80\80\80\08\05\01\04\0A\05\05\01\0C\00\12 \09\0E\00\0C\00\12\09\00\14\01\8A\80\80\80\08\09\01\0A\00\13\0E\00\14\02\87\80\80\80\08\0E\02\07\04\08\10\07\01\00\02" }>, section "__llvm_covfun", comdat, align 8
@__llvm_coverage_mapping = private constant { { i32, i32, i32, i32 }, [48 x i8] } { { i32, i32, i32, i32 } { i32 0, i32 48, i32 0, i32 5 }, [48 x i8] c"\02%-x\DA\13\D2\CF\C8\CFM\D5\CF\CBO\CC\D0/O,\CEM\CE/\13,I-.)\D6O\CBL\CA\CFKLN\CE\D4K\06\00\FA\F0\0D\91" }, section "__llvm_covmap", align 8
@__profc_main = private global [3 x i64] zeroinitializer, section "__llvm_prf_cnts", comdat, align 8
@__profd_main = private global { i64, i64, i64, i8*, i8*, i32, [2 x i16] } { i64 -2624081020897602054, i64 1245365802964056, i64 sub (i64 ptrtoint ([3 x i64]* @__profc_main to i64), i64 ptrtoint ({ i64, i64, i64, i8*, i8*, i32, [2 x i16] }* @__profd_main to i64)), i8* null, i8* null, i32 3, [2 x i16] zeroinitializer }, section "__llvm_prf_data", comdat($__profc_main), align 8
@__llvm_prf_nm = private constant [14 x i8] c"\04\0Cx\DA\CBM\CC\CC\03\00\04\1B\01\A6", section "__llvm_prf_names", align 1
@llvm.compiler.used = appending global [1 x i8*] [i8* bitcast ({ i64, i64, i64, i8*, i8*, i32, [2 x i16] }* @__profd_main to i8*)], section "llvm.metadata"
@llvm.used = appending global [3 x i8*] [i8* bitcast (<{ i64, i32, i64, i64, [98 x i8] }>* @__covrec_DB956436E78DD5FAu to i8*), i8* bitcast ({ { i32, i32, i32, i32 }, [48 x i8] }* @__llvm_coverage_mapping to i8*), i8* getelementptr inbounds ([14 x i8], [14 x i8]* @__llvm_prf_nm, i32 0, i32 0)], section "llvm.metadata"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 {
start:
  unreachable
}

; Function Attrs: nounwind
declare void @llvm.instrprof.increment(i8*, i64, i32, i32) #1

declare i32 @printf(i8* noundef, ...) #2

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { nounwind }
attributes #2 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }

!llvm.module.flags = !{!0, !1, !2, !3, !4, !5}
!llvm.ident = !{!6}

!0 = !{i32 2, !"EnableValueProfiling", i32 0}
!1 = !{i32 1, !"wchar_size", i32 4}
!2 = !{i32 7, !"PIC Level", i32 2}
!3 = !{i32 7, !"PIE Level", i32 2}
!4 = !{i32 7, !"uwtable", i32 1}
!5 = !{i32 7, !"frame-pointer", i32 2}
!6 = !{!"Ubuntu clang version 14.0.0-1ubuntu1.1"}
!7 = distinct !{!7, !8}
!8 = !{!"llvm.loop.mustprogress"}
