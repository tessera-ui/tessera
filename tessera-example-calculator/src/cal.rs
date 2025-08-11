use anyhow::anyhow;
use rsc::{Interpreter, parse, tokenize};

pub fn evaluate(input: &str, interpreter: &mut Interpreter<f64>) -> anyhow::Result<f64> {
    let tokens = tokenize(input).map_err(|e| anyhow!("{:?}", e))?;
    let expr = parse(&tokens).map_err(|e| anyhow!("{:?}", e))?;
    let result = interpreter.eval(&expr).map_err(|e| anyhow!("{:?}", e))?;
    Ok(result)
}
