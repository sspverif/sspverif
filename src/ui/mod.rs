use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::collections::HashMap;

#[cfg(test)]
pub(crate) mod mock;

pub(crate) trait ProofUI {
    fn new(num_proofs: u64) -> Self;
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

pub(crate) struct IndicatifProofUI {
    main_progress: MultiProgress,
    project_progress: Option<ProgressBar>,
    seq_proof_progress: HashMap<String, ProgressBar>,
    seq_proofstep_progress: HashMap<(String, String), ProgressBar>,
    seq_oracle_progress: HashMap<(String, String, String), ProgressBar>,
}

impl ProofUI for IndicatifProofUI {
    fn new(num_proofs: u64) -> Self {
        let main_progress = MultiProgress::new();
        let project_progress = if num_proofs > 1 {
            let project_progress = main_progress.add(ProgressBar::new(num_proofs));

            project_progress.set_style(indicatif_style::proof_bar());
            project_progress.set_message("Project");
            //project_progress.enable_steady_tick(std::time::Duration::from_secs(10));
            Some(project_progress)
        } else {
            None
        };

        IndicatifProofUI {
            main_progress,
            project_progress,
            seq_proof_progress: HashMap::new(),
            seq_proofstep_progress: HashMap::new(),
            seq_oracle_progress: HashMap::new(),
        }
    }

    fn println(&self, line: &str) -> std::io::Result<()> {
        self.main_progress.println(line)
    }

    fn start_proof(&mut self, proof_name: &str, num_proofsteps: u64) {
        let proof_progress = self.main_progress.add(ProgressBar::new(num_proofsteps));

        proof_progress.set_style(indicatif_style::proof_bar());
        proof_progress.set_message(format!("{proof_name}"));
        //proof_progress.enable_steady_tick(std::time::Duration::from_secs(10));

        self.seq_proof_progress
            .insert(proof_name.to_string(), proof_progress);
    }

    fn finish_proof(&mut self, proof_name: &str) {
        if let Some(project_progress) = &self.project_progress {
            project_progress.inc(1);
        };
    }

    fn start_proofstep(&mut self, proof_name: &str, proofstep_name: &str) {
        let proofstep_progress = self.main_progress.add(ProgressBar::new(1));
        proofstep_progress.set_style(indicatif_style::proofstep_bar());
        proofstep_progress.set_message(format!("{proofstep_name}"));
        //proofstep_progress.enable_steady_tick(std::time::Duration::from_secs(10));

        self.seq_proofstep_progress.insert(
            (proof_name.to_string(), proofstep_name.to_string()),
            proofstep_progress,
        );
    }

    fn proofstep_is_reduction(&mut self, proof_name: &str, proofstep_name: &str) {
        if let Some(proofstep_progress) = self
            .seq_proofstep_progress
            .get(&(proof_name.to_string(), proofstep_name.to_string()))
        {
            proofstep_progress.set_length(1);
            proofstep_progress.inc(1);
            proofstep_progress.tick();
        } else {
            unreachable!("{proof_name} -- {proofstep_name}");
        };
    }

    fn proofstep_set_oracles(&mut self, proof_name: &str, proofstep_name: &str, num_oracles: u64) {
        if let Some(proofstep_progress) = self
            .seq_proofstep_progress
            .get(&(proof_name.to_string(), proofstep_name.to_string()))
        {
            proofstep_progress.set_length(num_oracles);
            proofstep_progress.tick();
        } else {
            unreachable!("{proof_name} -- {proofstep_name}");
        };
    }

    fn finish_proofstep(&mut self, proof_name: &str, proofstep_name: &str) {
        if let Some(proof_progress) = self.seq_proof_progress.iter().find_map(|(name, progress)| {
            if proof_name == name {
                Some(progress)
            } else {
                None
            }
        }) {
            proof_progress.inc(1);
        };

        self.seq_oracle_progress
            .retain(|k, _v| !(k.0 == proof_name && k.1 == proofstep_name))
    }

    fn start_oracle(
        &mut self,
        proof_name: &str,
        proofstep_name: &str,
        oracle_name: &str,
        num_lemmata: u64,
    ) {
        let oracle_progress = self.main_progress.add(ProgressBar::new(num_lemmata));
        oracle_progress.set_style(indicatif_style::oracle_bar());
        oracle_progress.set_message(format!("{oracle_name}"));
        //oracle_progress.enable_steady_tick(std::time::Duration::from_secs(10));

        self.seq_oracle_progress.insert(
            (
                proof_name.to_string(),
                proofstep_name.to_string(),
                oracle_name.to_string(),
            ),
            oracle_progress,
        );
    }

    fn finish_oracle(&mut self, proof_name: &str, proofstep_name: &str, oracle_name: &str) {
        if let Some(proofstep_progress) = self
            .seq_proofstep_progress
            .get(&(proof_name.to_string(), proofstep_name.to_string()))
        {
            proofstep_progress.inc(1);
        };
    }

    fn start_lemma(
        &mut self,
        proof_name: &str,
        proofstep_name: &str,
        oracle_name: &str,
        lemma_name: &str,
    ) {
        if let Some(oracle_progress) = self.seq_oracle_progress.get(&(
            proof_name.to_string(),
            proofstep_name.to_string(),
            oracle_name.to_string(),
        )) {
            oracle_progress.set_message(format!("{oracle_name} (cur: {lemma_name})"));
        }
    }

    fn finish_lemma(
        &mut self,
        proof_name: &str,
        proofstep_name: &str,
        oracle_name: &str,
        lemma_name: &str,
    ) {
        if let Some(oracle_progress) = self.seq_oracle_progress.get(&(
            proof_name.to_string(),
            proofstep_name.to_string(),
            oracle_name.to_string(),
        )) {
            oracle_progress.inc(1);
            oracle_progress.set_message(format!("{oracle_name}"));
        }
    }
}

mod indicatif_style {
    use indicatif::ProgressStyle;

    pub(super) fn proof_bar() -> ProgressStyle {
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:80.cyan/blue} {pos:>3}/{len:3} {msg}",
        )
        .unwrap()
        .progress_chars("#>-")
    }

    pub(super) fn proofstep_bar() -> ProgressStyle {
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:80.yellow/white} {pos:>3}/{len:3} {msg}",
        )
        .unwrap()
        .progress_chars("#>-")
    }

    pub(super) fn oracle_bar() -> ProgressStyle {
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:80.magenta/white} {pos:>3}/{len:3} {msg}",
        )
        .unwrap()
        .progress_chars("#>-")
    }
}
