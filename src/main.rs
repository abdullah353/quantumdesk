use quantumdesk::QuantumDesk;

fn main() -> anyhow::Result<()> {
    quantumdesk::run(QuantumDesk::default())
}
