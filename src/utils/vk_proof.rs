// AUTO-GENERATED FROM snarkjs
// DO NOT EDIT MANUALLY
#![allow(non_snake_case)]
#![allow(dead_code)]

use ark_bn254::{Fq, Fq2, Fr, G1Affine, G1Projective, G2Affine, G2Projective};
use ark_std::{str::FromStr, vec::Vec, Zero};

pub struct Vk {
    pub alpha_g1: G1Projective,
    pub beta_g2: G2Projective,
    pub gamma_g2: G2Projective,
    pub delta_g2: G2Projective,
    pub ic: Vec<G1Projective>,
}

pub fn verification_key() -> Vk {
    Vk {
        alpha_g1: G1Affine::new(
            Fq::from_str("1").unwrap(),
            Fq::from_str("2").unwrap(),
            false,
        )
        .into(),
        beta_g2: G2Affine::new(
            Fq2::new(
                Fq::from_str(
                    "10857046999023057135944570762232829481370756359578518086990519993285655852781",
                )
                .unwrap(),
                Fq::from_str(
                    "11559732032986387107991004021392285783925812861821192530917403151452391805634",
                )
                .unwrap(),
            ),
            Fq2::new(
                Fq::from_str(
                    "8495653923123431417604973247489272438418190587263600148770280649306958101930",
                )
                .unwrap(),
                Fq::from_str(
                    "4082367875863433681332203403145435568316851327593401208105741076214120093531",
                )
                .unwrap(),
            ),
            false,
        )
        .into(),
        gamma_g2: G2Affine::new(
            Fq2::new(
                Fq::from_str(
                    "10857046999023057135944570762232829481370756359578518086990519993285655852781",
                )
                .unwrap(),
                Fq::from_str(
                    "11559732032986387107991004021392285783925812861821192530917403151452391805634",
                )
                .unwrap(),
            ),
            Fq2::new(
                Fq::from_str(
                    "8495653923123431417604973247489272438418190587263600148770280649306958101930",
                )
                .unwrap(),
                Fq::from_str(
                    "4082367875863433681332203403145435568316851327593401208105741076214120093531",
                )
                .unwrap(),
            ),
            false,
        )
        .into(),
        delta_g2: G2Affine::new(
            Fq2::new(
                Fq::from_str(
                    "10857046999023057135944570762232829481370756359578518086990519993285655852781",
                )
                .unwrap(),
                Fq::from_str(
                    "11559732032986387107991004021392285783925812861821192530917403151452391805634",
                )
                .unwrap(),
            ),
            Fq2::new(
                Fq::from_str(
                    "8495653923123431417604973247489272438418190587263600148770280649306958101930",
                )
                .unwrap(),
                Fq::from_str(
                    "4082367875863433681332203403145435568316851327593401208105741076214120093531",
                )
                .unwrap(),
            ),
            false,
        )
        .into(),
        ic: ark_std::vec![
            G1Affine::new(
                Fq::from_str("1").unwrap(),
                Fq::from_str(
                    "21888242871839275222246405745257275088696311157297823662689037894645226208581"
                )
                .unwrap(),
                false
            )
            .into(),
            G1Projective::zero()
        ],
    }
}

pub struct Proof {
    pub a: G1Projective,
    pub b: G2Projective,
    pub c: G1Projective,
}

pub fn sample_proof() -> (Proof, Vec<Fr>) {
    let proof = Proof {
        a: G1Affine::new(
            Fq::from_str(
                "5846067817534501029168219758664959042318162400101284663795922097724883505222",
            )
            .unwrap(),
            Fq::from_str(
                "10976197405737721333488952110323761168740735339378490158153265794290976899646",
            )
            .unwrap(),
            false,
        )
        .into(),
        b: G2Affine::new(
            Fq2::new(
                Fq::from_str(
                    "5941609983153469020012280960721666409926273015922154969498594507087015854680",
                )
                .unwrap(),
                Fq::from_str(
                    "8836719669744972275721198312956446745168340442154705538528481296230285294558",
                )
                .unwrap(),
            ),
            Fq2::new(
                Fq::from_str(
                    "3480701898224563411932641606285736377482641858927593172848682636984436244374",
                )
                .unwrap(),
                Fq::from_str(
                    "489700308443953829354817105151096139602860112340426234009604985073453462233",
                )
                .unwrap(),
            ),
            false,
        )
        .into(),
        c: G1Affine::new(
            Fq::from_str(
                "11099538397044349801466183978400408533965706819252075854811832834022244673239",
            )
            .unwrap(),
            Fq::from_str(
                "7118650027467702224847974637409557263182673231661819166195649449954026323460",
            )
            .unwrap(),
            false,
        )
        .into(),
    };
    let public_inputs = ark_std::vec![Fr::from_str(
        "5719944538356554403817391916218209963465482684641156206505489751898082045988"
    )
    .unwrap()];
    (proof, public_inputs)
}
