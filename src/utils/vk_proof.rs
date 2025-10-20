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
                "20216596021337674102130712069355770283071184329735499582367767855933738996145",
            )
            .unwrap(),
            Fq::from_str(
                "6102033962155836056264707840840638985124255261357259900635309157743423918468",
            )
            .unwrap(),
            false,
        )
        .into(),
        b: G2Affine::new(
            Fq2::new(
                Fq::from_str(
                    "19626852365668056526927066154617809168781999839420620815895849451889959716348",
                )
                .unwrap(),
                Fq::from_str(
                    "11137125786241859477020899599745758351205566087668595260682734703419264370653",
                )
                .unwrap(),
            ),
            Fq2::new(
                Fq::from_str(
                    "19133139566602697557675441918469625448941956458159074568667706430253513439005",
                )
                .unwrap(),
                Fq::from_str(
                    "19240758422973115914612893855350240511110524778403060014339985061917147150043",
                )
                .unwrap(),
            ),
            false,
        )
        .into(),
        c: G1Affine::new(
            Fq::from_str(
                "5077626855977500568272553282783382150109021272790448474987903502364259599269",
            )
            .unwrap(),
            Fq::from_str(
                "15588186921503631277034386661498693710425851048235025765121170467313529140575",
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
