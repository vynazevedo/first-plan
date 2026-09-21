//! Harnesses import the implementation compiled into first-plan-core.
#[path = "../../engine/crates/core/src/invariants.rs"]
pub mod invariants;

#[cfg(kani)]
mod proofs {
    use super::invariants::*;

    #[kani::proof]
    fn budget_never_overflows_or_exceeds_limit() {
        let used: usize = kani::any();
        let cost: usize = kani::any();
        let budget: usize = kani::any();
        let result = reserve_budget(used, cost, budget);
        // Independent specification uses wider arithmetic on the pinned 64-bit target.
        let sum = used as u128 + cost as u128;
        assert_eq!(result.is_some(), sum <= budget as u128);
        if let Some(next) = result {
            assert_eq!(next as u128, sum);
            assert!(next <= budget);
        }
    }

    #[kani::proof]
    fn strict_gate_rejects_missing_evidence() {
        let strict: bool = kani::any();
        let complete: bool = kani::any();
        let breaking: usize = kani::any();
        let accepted = contract_gate(strict, complete, breaking);
        if strict {
            if accepted {
                assert!(complete);
                assert_eq!(breaking, 0);
            }
            if complete && breaking == 0 {
                assert!(accepted);
            }
        } else {
            assert!(accepted);
        }
    }

    #[kani::proof]
    #[kani::unwind(6)]
    fn replacement_preserves_surrounding_bytes() {
        // Exhaustive within these explicit bounds: 0..4 ASCII bytes per string.
        // UTF-8 beyond ASCII is covered by tests, not claimed by this proof.
        let raw: [u8; 4] = kani::any();
        let replacement: [u8; 4] = kani::any();
        let raw = raw.map(|b| b & 0x7f);
        let replacement = replacement.map(|b| b & 0x7f);
        let len = (kani::any::<u8>() % 5) as usize;
        let block_len = (kani::any::<u8>() % 5) as usize;
        let start = (kani::any::<u8>() % 6) as usize;
        let end = (kani::any::<u8>() % 6) as usize;
        let source_full = std::str::from_utf8(&raw).unwrap();
        let block_full = std::str::from_utf8(&replacement).unwrap();
        let source = &source_full[..len];
        let block = &block_full[..block_len];
        let result = replace_range(source, start, end, block);
        if start > end || end > len {
            assert!(result.is_none());
        } else {
            let result = result.unwrap();
            let bytes = result.as_bytes();
            assert_eq!(bytes.len(), start + block_len + len - end);
            for i in 0..start {
                assert_eq!(bytes[i], raw[i]);
            }
            for i in 0..block_len {
                assert_eq!(bytes[start + i], replacement[i]);
            }
            for i in end..len {
                assert_eq!(bytes[start + block_len + i - end], raw[i]);
            }
        }
    }
}
