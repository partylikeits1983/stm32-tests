use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize)]
struct VerificationKey {
    #[serde(rename = "vk_alpha_1")]
    vk_alpha_1: Vec<String>,
    #[serde(rename = "vk_beta_2")]
    vk_beta_2: Vec<Vec<String>>,
    #[serde(rename = "vk_gamma_2")]
    vk_gamma_2: Vec<Vec<String>>,
    #[serde(rename = "vk_delta_2")]
    vk_delta_2: Vec<Vec<String>>,
    #[serde(rename = "IC")]
    ic: Vec<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct Proof {
    pi_a: Vec<String>,
    pi_b: Vec<Vec<String>>,
    pi_c: Vec<String>,
}

fn g1_to_rust(p: &[String]) -> String {
    // snarkjs format: [x, y, z] where z=1 for affine, z=0 for infinity
    // We only use affine coordinates, so we check z
    if p.len() >= 3 && p[2] == "0" {
        // Point at infinity
        return "G1Projective::zero()".to_string();
    }
    
    // Regular affine point (z should be 1)
    format!(
        "G1Affine::new(\n      Fq::from_str(\"{}\").unwrap(),\n      Fq::from_str(\"{}\").unwrap(),\n      false\n    ).into()",
        p[0], p[1]
    )
}

fn g2_to_rust(p: &[Vec<String>]) -> String {
    // snarkjs format: [[x_c0, x_c1], [y_c0, y_c1], [z_c0, z_c1]]
    // Check if point at infinity (z = [0, 0])
    if p.len() >= 3 && p[2][0] == "0" && p[2][1] == "0" {
        return "G2Projective::zero()".to_string();
    }
    
    format!(
        "G2Affine::new(\n      Fq2::new(\n        Fq::from_str(\"{}\").unwrap(),\n        Fq::from_str(\"{}\").unwrap()\n      ),\n      Fq2::new(\n        Fq::from_str(\"{}\").unwrap(),\n        Fq::from_str(\"{}\").unwrap()\n      ),\n      false\n    ).into()",
        p[0][0], p[0][1], p[1][0], p[1][1]
    )
}

fn fr_to_rust(x: &str) -> String {
    format!("Fr::from_str(\"{}\").unwrap()", x)
}

fn main() {
    println!("Reading snarkjs output files...");

    // Read verification key
    let vk_json = fs::read_to_string("../vk.json")
        .expect("Failed to read vk.json. Run setup.sh first.");
    let vk: VerificationKey = serde_json::from_str(&vk_json)
        .expect("Failed to parse vk.json");

    // Read proof
    let proof_json = fs::read_to_string("../proof.json")
        .expect("Failed to read proof.json. Run generate_proof.sh first.");
    let proof: Proof = serde_json::from_str(&proof_json)
        .expect("Failed to parse proof.json");

    // Read public inputs
    let public_json = fs::read_to_string("../public.json")
        .expect("Failed to read public.json. Run generate_proof.sh first.");
    let public_inputs: Vec<String> = serde_json::from_str(&public_json)
        .expect("Failed to parse public.json");

    println!("Generating Rust code...");

    // Generate IC array
    let ic_elements: Vec<String> = vk.ic.iter()
        .map(|p| g1_to_rust(p))
        .collect();
    let ic_array = ic_elements.join(",\n    ");

    // Generate public inputs array
    let public_elements: Vec<String> = public_inputs.iter()
        .map(|x| fr_to_rust(x))
        .collect();
    let public_array = public_elements.join(",\n    ");

    let output = format!(
r#"// AUTO-GENERATED FROM snarkjs
// DO NOT EDIT MANUALLY
#![allow(non_snake_case)]
#![allow(dead_code)]

use ark_bn254::{{Fq, Fq2, Fr, G1Affine, G1Projective, G2Affine, G2Projective}};
use ark_std::{{str::FromStr, vec::Vec, Zero}};

pub struct Vk {{
  pub alpha_g1: G1Projective,
  pub beta_g2: G2Projective,
  pub gamma_g2: G2Projective,
  pub delta_g2: G2Projective,
  pub ic: Vec<G1Projective>,
}}

pub fn verification_key() -> Vk {{
  Vk {{
    alpha_g1: {},
    beta_g2: {},
    gamma_g2: {},
    delta_g2: {},
    ic: ark_std::vec![
      {}
    ],
  }}
}}

pub struct Proof {{
  pub a: G1Projective,
  pub b: G2Projective,
  pub c: G1Projective,
}}

pub fn sample_proof() -> (Proof, Vec<Fr>) {{
  let proof = Proof {{
    a: {},
    b: {},
    c: {},
  }};
  let public_inputs = ark_std::vec![
    {}
  ];
  (proof, public_inputs)
}}
"#,
        g1_to_rust(&vk.vk_alpha_1),
        g2_to_rust(&vk.vk_beta_2),
        g2_to_rust(&vk.vk_gamma_2),
        g2_to_rust(&vk.vk_delta_2),
        ic_array,
        g1_to_rust(&proof.pi_a),
        g2_to_rust(&proof.pi_b),
        g1_to_rust(&proof.pi_c),
        public_array
    );

    let output_path = "../../src/utils/vk_proof.rs";
    fs::write(output_path, output)
        .expect("Failed to write output file");

    println!("✓ Generated {}", output_path);
    println!();
    println!("Next steps:");
    println!("  1. cargo build --release --bin groth16_demo");
    println!("  2. probe-rs run --chip STM32F411CEUx target/thumbv7em-none-eabihf/release/groth16_demo");
}