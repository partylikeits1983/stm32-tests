//! Groth16 proof verification utilities for STM32
//!
//! This module provides a wrapper around arkworks Groth16 verification
//! optimized for embedded systems (no_std).

#![allow(dead_code)]

use ark_bn254::{Bn254, Fr, G1Affine, G1Projective, G2Affine, G2Projective};
use ark_ec::{AffineCurve, PairingEngine, ProjectiveCurve};
use ark_std::vec::Vec;
use core::ops::{AddAssign, Neg};

/// Re-export the generated verification key and proof
#[path = "vk_proof.rs"]
pub mod vk_proof;

/// Wrapper for Groth16 proof that matches our generated format
pub struct Proof {
    pub a: G1Projective,
    pub b: G2Projective,
    pub c: G1Projective,
}

/// Wrapper for verification key
pub struct Vk<'a> {
    pub alpha_g1: G1Projective,
    pub beta_g2: G2Projective,
    pub gamma_g2: G2Projective,
    pub delta_g2: G2Projective,
    pub ic: &'a [G1Projective],
}

/// Prepared verification key for faster verification
pub struct PreparedVk {
    pub alpha_g1: G1Affine,
    pub beta_g2: G2Affine,
    pub gamma_g2_neg: G2Affine,
    pub delta_g2_neg: G2Affine,
    pub gamma_abc_g1: Vec<G1Affine>,
    pub e_alpha_beta: <Bn254 as PairingEngine>::Fqk,
}

impl<'a> Vk<'a> {
    /// Prepare the verification key for faster verification
    pub fn prepare(&self) -> PreparedVk {
        let alpha_affine = self.alpha_g1.into_affine();
        let beta_affine = self.beta_g2.into_affine();

        PreparedVk {
            alpha_g1: alpha_affine,
            beta_g2: beta_affine,
            gamma_g2_neg: self.gamma_g2.into_affine().neg(),
            delta_g2_neg: self.delta_g2.into_affine().neg(),
            gamma_abc_g1: self.ic.iter().map(|p| p.into_affine()).collect(),
            e_alpha_beta: Bn254::pairing(alpha_affine, beta_affine),
        }
    }
}

/// Aggregate public inputs with IC elements
fn aggregate_inputs(prep_vk: &PreparedVk, public_inputs: &[Fr]) -> G1Projective {
    if (public_inputs.len() + 1) != prep_vk.gamma_abc_g1.len() {
        // Return identity element
        return prep_vk.gamma_abc_g1[0].into_projective();
    }

    let mut g_ic = prep_vk.gamma_abc_g1[0].into_projective();
    for (input, ic_point) in public_inputs
        .iter()
        .zip(prep_vk.gamma_abc_g1.iter().skip(1))
    {
        use ark_ff::PrimeField;
        g_ic.add_assign(&ic_point.mul(input.into_repr()));
    }

    g_ic
}

/// Verify a Groth16 proof
///
/// # Arguments
/// * `vk` - The verification key
/// * `proof` - The proof to verify
/// * `public_inputs` - The public inputs (should match the circuit's public signals)
///
/// # Returns
/// * `Ok(())` if the proof is valid
/// * `Err(())` if the proof is invalid
pub fn verify_proof(vk: &Vk<'_>, proof: &Proof, public_inputs: &[Fr]) -> Result<(), ()> {
    let pvk = vk.prepare();
    verify_proof_prepared(&pvk, proof, public_inputs)
}

/// Verify a Groth16 proof with a pre-prepared verification key
///
/// This is more efficient if you're verifying multiple proofs with the same key.
///
/// # Arguments
/// * `pvk` - The prepared verification key
/// * `proof` - The proof to verify
/// * `public_inputs` - The public inputs
///
/// # Returns
/// * `Ok(())` if the proof is valid
/// * `Err(())` if the proof is invalid
pub fn verify_proof_prepared(
    pvk: &PreparedVk,
    proof: &Proof,
    public_inputs: &[Fr],
) -> Result<(), ()> {
    // Aggregate public inputs
    let g_ic = aggregate_inputs(pvk, public_inputs);

    // Convert proof to affine
    let proof_a = proof.a.into_affine();
    let proof_b = proof.b.into_affine();
    let proof_c = proof.c.into_affine();
    let g_ic_affine = g_ic.into_affine();

    // Groth16 verification equation:
    // e(A, B) = e(alpha, beta) * e(g_ic, gamma) * e(C, delta)
    //
    // Rearranged as:
    // e(A, B) * e(g_ic, -gamma) * e(C, -delta) = e(alpha, beta)

    // Compute pairings
    let e_a_b = Bn254::pairing(proof_a, proof_b);
    let e_ic_gamma = Bn254::pairing(g_ic_affine, pvk.gamma_g2_neg);
    let e_c_delta = Bn254::pairing(proof_c, pvk.delta_g2_neg);

    // Compute LHS: e(A, B) * e(g_ic, -gamma) * e(C, -delta)
    use core::ops::MulAssign;
    let mut lhs = e_a_b;
    lhs.mul_assign(&e_ic_gamma);
    lhs.mul_assign(&e_c_delta);

    // Check if LHS == RHS (e_alpha_beta)
    if lhs == pvk.e_alpha_beta {
        Ok(())
    } else {
        Err(())
    }
}
