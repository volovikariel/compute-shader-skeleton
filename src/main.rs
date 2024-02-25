// mod minimal;
mod repeatable;

async fn run() {
    use repeatable::Runner;
    let runner = Runner::new().await;

    let inputs: [&[f32]; 3] = [&[1.0, 2.0], &[10.0, -12.0], &[1.0]];
    // let inputs = [[1.0, 2.0]];
    for input in inputs {
        let output = runner.run(input).await;
        println!("{input:?} -> {output:?}");
    }
}

fn main() {
    pollster::block_on(run());
}
