pub(crate) mod indicatif;
#[cfg(test)]
pub(crate) mod mock;

pub(crate) trait ProofUI {
    fn println(&self, line: &str) -> std::io::Result<()>;

    fn start_proof(&mut self, proof_name: &str, num_proofsteps: u64);

    fn finish_proof(&mut self, proof_name: &str);

    fn start_proofstep(&mut self, proof_name: &str, proofstep_name: &str);

    fn proofstep_is_reduction(&mut self, proof_name: &str, proofstep_name: &str);

    fn proofstep_set_oracles(&mut self, proof_name: &str, proofstep_name: &str, num_oracles: u64);

    fn finish_proofstep(&mut self, proof_name: &str, proofstep_name: &str);

    fn start_oracle(
        &mut self,
        proof_name: &str,
        proofstep_name: &str,
        oracle_name: &str,
        num_lemmata: u64,
    );

    fn finish_oracle(&mut self, proof_name: &str, proofstep_name: &str, oracle_name: &str);

    fn start_lemma(
        &mut self,
        proof_name: &str,
        proofstep_name: &str,
        oracle_name: &str,
        lemma_name: &str,
    );

    fn finish_lemma(
        &mut self,
        proof_name: &str,
        proofstep_name: &str,
        oracle_name: &str,
        lemma_name: &str,
    );
}
