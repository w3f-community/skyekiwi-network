mod script;

use crate::script::Script;
use clap::Clap;
// use skw_vm_host::VMOutcome;
// use skw_vm_host::{mocks::mock_external::Receipt};
use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeMap,
    {Deserialize, Deserializer, Serialize, Serializer},
};

use std::path::PathBuf;
use std::{collections::HashMap, fmt, fs};

#[derive(Debug, Clone)]
struct State(HashMap<Vec<u8>, Vec<u8>>);

impl Serialize for State {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (k, v) in &self.0 {
            map.serialize_entry(&base64::encode(&k).to_string(), &base64::encode(&v).to_string())?;
        }
        map.end()
    }
}

struct Base64HashMapVisitor;

impl<'de> Visitor<'de> for Base64HashMapVisitor {
    type Value = State;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Base64 serialized HashMap")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = HashMap::with_capacity(access.size_hint().unwrap_or(0));

        while let Some((key, value)) = access.next_entry::<String, String>()? {
            map.insert(base64::decode(&key).unwrap(), base64::decode(&value).unwrap());
        }

        Ok(State(map))
    }
}

impl<'de> Deserialize<'de> for State {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(Base64HashMapVisitor {})
    }
}

#[derive(Clap)]
struct CliArgs {
    /// Specifies the execution context in JSON format, see `VMContext`.
    #[clap(long)]
    context: Option<String>,
    /// Reads the context from the file.
    #[clap(long)]
    context_file: Option<PathBuf>,
    /// Overrides input field of the context with the given string.
    #[clap(long)]
    input: Option<String>,
    /// The name of the method to call on the smart contract.
    #[clap(long)]
    method_name: String,
    /// Key-value state in JSON base64 format for the smart contract as HashMap.
    #[clap(long)]
    state: Option<String>,
    /// Reads the state from the file
    #[clap(long)]
    state_file: Option<PathBuf>,
    /// If the contract should be called by a callback or several callbacks you
    /// can pass result of executing functions that trigger the callback. For
    /// non-callback calls it can be omitted.
    #[clap(long)]
    promise_results: Vec<String>,
    /// Specifies the economics and Wasm config in JSON format, see `Config`.
    #[clap(long)]
    config: Option<String>,
    /// Reads the config from the file.
    #[clap(long)]
    config_file: Option<PathBuf>,
    /// File path that contains the Wasm code to run.
    #[clap(long)]
    wasm_file: PathBuf,
    /// Prints execution times of various components.
    #[clap(long)]
    timings: bool,
}

// #[derive(Debug, Clone)]
// struct StandaloneOutput {
//     pub outcome: Option<VMOutcome>,
//     pub err: Option<String>,
//     pub receipts: Vec<Receipt>,
//     pub state: State,
// }

fn main() {
    let cli_args = CliArgs::parse();

    if cli_args.timings {
        tracing_span_tree::span_tree().enable();
    }

    let mut script = Script::default();

    if let Some(config) = &cli_args.config {
        script.vm_config(serde_json::from_str(config).unwrap());
    }
    if let Some(path) = &cli_args.config_file {
        script.vm_config_from_file(path);
    }

    if let Some(state_str) = &cli_args.state {
        script.initial_state(serde_json::from_str(state_str).unwrap());
    }
    if let Some(path) = &cli_args.state_file {
        script.initial_state_from_file(path);
    }

    let code = fs::read(&cli_args.wasm_file).unwrap();
    let contract = script.contract(code);

    let step = script.step(contract, &cli_args.method_name);

    if let Some(value) = &cli_args.context {
        step.context(serde_json::from_str(value).unwrap());
    }
    if let Some(path) = &cli_args.context_file {
        step.context_from_file(path);
    }

    if let Some(value) = cli_args.input {
        step.input(value.as_bytes().to_vec());
    }

    let promise_results =
        cli_args.promise_results.iter().map(|it| serde_json::from_str(it).unwrap()).collect();
    step.promise_results(promise_results);

    let mut results = script.run();
    let (outcome, err) = results.outcomes.pop().unwrap();

    // println!(
    //     "{:#?}",
    //     &StandaloneOutput {
    //         outcome: outcome.clone(),
    //         err: err.map(|it| it.to_string()),
    //         receipts: results.state.get_receipt_create_calls().clone(),
    //         state: State(results.state.fake_trie.clone()),
    //     }
    // );
    
    println!("{}", "{");
    println!("\"outcome\": \"{:?}\",", outcome.clone().unwrap());

    println!(
        "\"state\": {:?},",
        serde_json::to_string(&State(results.state.fake_trie)).unwrap()
    );

    println!("\"error\": \"{:?}\"", err.map(|it| {
        it.to_string()
    }));

    println!("{}", "}")
    // match &outcome {
    //     Some(outcome) => {
    //         println!("{:#?}", outcome.profile);
    //     }
    //     _ => {}
    // }
}
