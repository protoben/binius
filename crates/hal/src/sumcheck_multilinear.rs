// Copyright 2024-2025 Irreducible Inc.

use binius_field::PackedField;
use binius_math::MultilinearPoly;

/// An individual multilinear polynomial in a multivariate composite.
#[derive(Debug, Clone)]
pub enum SumcheckMultilinear<P, M>
where
	P: PackedField,
	M: MultilinearPoly<P>,
{
	/// Small field multilinear - to be folded into large field at `switchover` round
	/// A suffix of constant scalars at the end is provided via `const_suffix` hint.
	Transparent {
		multilinear: M,
		switchover_round: usize,
		const_suffix: (P::Scalar, usize),
	},
	/// Large field multilinear - halved in size each round
	/// If length is less than expected one for the current round, pad with `suffix_eval` scalars.
	Folded {
		large_field_folded_evals: Vec<P>,
		suffix_eval: P::Scalar,
	},
}

impl<P: PackedField, M: MultilinearPoly<P>> SumcheckMultilinear<P, M> {
	pub fn transparent(multilinear: M, switchover_fn: &impl Fn(usize) -> usize) -> Self {
		let switchover_round = (*switchover_fn)(1 << multilinear.log_extension_degree());

		Self::Transparent {
			multilinear,
			switchover_round,
			const_suffix: Default::default(),
		}
	}

	pub fn folded(large_field_folded_evals: Vec<P>) -> Self {
		Self::Folded {
			large_field_folded_evals,
			suffix_eval: Default::default(),
		}
	}

	pub fn const_suffix(&self, n_vars: usize) -> (P::Scalar, usize) {
		match self {
			Self::Transparent { const_suffix, .. } => *const_suffix,
			Self::Folded {
				large_field_folded_evals,
				suffix_eval,
			} => (
				*suffix_eval,
				(1usize << n_vars).saturating_sub(large_field_folded_evals.len() << P::LOG_WIDTH),
			),
		}
	}

	pub fn update_const_suffix(&mut self, n_vars: usize, new_const_suffix: (P::Scalar, usize)) {
		match self {
			Self::Transparent { const_suffix, .. } => {
				*const_suffix = new_const_suffix;
			}

			Self::Folded {
				large_field_folded_evals,
				suffix_eval,
			} => {
				let (new_suffix_eval, new_suffix_len) = new_const_suffix;
				*suffix_eval = new_suffix_eval;
				large_field_folded_evals
					.truncate(((1 << n_vars) - new_suffix_len).div_ceil(P::WIDTH));
			}
		}
	}
}
