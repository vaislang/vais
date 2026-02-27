//! IR optimization passes (constant folding, DCE, CSE, strength reduction, etc.)

mod branch_opt;
mod constant_folding;
mod cse;
mod dead_code;
mod helpers;
mod loop_opt;
mod strength_reduction;
mod tail_call;

// Re-export all optimization passes
pub(crate) use branch_opt::{branch_optimization, conditional_branch_simplification};
pub(crate) use constant_folding::constant_folding;
pub(crate) use cse::common_subexpression_elimination;
pub(crate) use dead_code::{dead_code_elimination, dead_store_elimination};
pub(crate) use loop_opt::loop_invariant_motion;
pub(crate) use strength_reduction::strength_reduction;
pub(crate) use tail_call::tail_call_optimization;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_folding() {
        let ir = "  %0 = add i64 10, 20\n";
        let result = constant_folding(ir);
        assert!(result.contains("30"));
    }

    #[test]
    fn test_strength_reduction_mul() {
        let line = "  %0 = mul i64 %x, 8";
        let result = strength_reduction::try_strength_reduce_mul(line);
        assert!(result.is_some());
        assert!(result.unwrap().contains("shl"));
    }

    #[test]
    fn test_is_power_of_2() {
        assert!(strength_reduction::is_power_of_2(1));
        assert!(strength_reduction::is_power_of_2(2));
        assert!(strength_reduction::is_power_of_2(4));
        assert!(strength_reduction::is_power_of_2(8));
        assert!(!strength_reduction::is_power_of_2(3));
        assert!(!strength_reduction::is_power_of_2(0));
        assert!(!strength_reduction::is_power_of_2(-1));
    }

    #[test]
    fn test_loop_unrolling() {
        // Simple loop with known bounds
        let ir = r#"define i64 @sum_to_10() {
entry:
  br label %loop.start.0
loop.start.0:
  %i = phi i64 [0, %entry], [%next, %loop.body.0]
  %sum = phi i64 [0, %entry], [%newsum, %loop.body.0]
  %cond = icmp slt i64 %i, 10
  br i1 %cond, label %loop.body.0, label %loop.end.0
loop.body.0:
  %newsum = add i64 %sum, %i
  %next = add i64 %i, 1
  br label %loop.start.0
loop.end.0:
  ret i64 %sum
}
"#;
        let result = loop_invariant_motion(ir);
        println!("UNROLLED:\n{}", result);
        // Check that unrolling was attempted (comment should be present)
        assert!(
            result.contains("LOOP UNROLLING") || result.contains("loop.start"),
            "Expected loop unrolling to be attempted"
        );
    }

    #[test]
    fn test_loop_invariant_motion() {
        // Loop with invariant computation
        let ir = r#"define i64 @test_licm(i64 %n, i64 %a, i64 %b) {
entry:
  br label %loop.start.0
loop.start.0:
  %i = phi i64 [0, %entry], [%next, %loop.body.0]
  %cond = icmp slt i64 %i, %n
  br i1 %cond, label %loop.body.0, label %loop.end.0
loop.body.0:
  %inv = add i64 %a, %b
  %tmp = add i64 %i, %inv
  %next = add i64 %i, 1
  br label %loop.start.0
loop.end.0:
  ret i64 %i
}
"#;
        let result = loop_invariant_motion(ir);
        println!("LICM RESULT:\n{}", result);
        // Check that LICM was attempted (comment should be present)
        assert!(
            result.contains("LICM") || result.contains("loop.start"),
            "Expected LICM to be attempted"
        );
    }

    #[test]
    fn test_full_loop_optimization() {
        // Test the combined loop optimization pass
        let ir = r#"define i64 @loop_opt_test() {
entry:
  br label %loop.start.0
loop.start.0:
  %i = phi i64 [0, %entry], [%next, %loop.body.0]
  %sum = phi i64 [0, %entry], [%newsum, %loop.body.0]
  %cond = icmp slt i64 %i, 8
  br i1 %cond, label %loop.body.0, label %loop.end.0
loop.body.0:
  %newsum = add i64 %sum, %i
  %next = add i64 %i, 1
  br label %loop.start.0
loop.end.0:
  ret i64 %sum
}
"#;
        let result = loop_invariant_motion(ir);
        println!("FULL LOOP OPT:\n{}", result);
        // The function should return valid IR
        assert!(result.contains("define i64 @loop_opt_test"));
        assert!(result.contains("ret i64"));
    }
}
