//! `kanso-acp-fake <script> [answer]`: the fake ACP agent over stdio, for
//! tests that launch an agent the way Kanso launches a real one.

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let script = kanso_acp::fake::Script::from_args(&args);
    kanso_acp::fake::serve(std::io::stdin(), std::io::stdout(), script);
}
