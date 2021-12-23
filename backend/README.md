# Backend

API documentation of backend can be found at [docs.thuburrow.com](https://docs/.thburrow.com)

## Convention

1. Please checkout a new branch from `backend` to develop your new feature or fix the bug. Recommended to give your branch a name relevant to what you are doing. e.g. `feature/login` or `fix/email`.
2. Rust coding style convention: [coding-style](https://wiki.jikexueyuan.com/project/rust-primer/coding-style/style.html)
3. Please run `cargo clippy` and `cargo fmt` before submit your code to lint and format your code.

## Deployment

- Run `openssl rand -base64 32` to generate `ROCKET_SECRET_KEY`.
- Copy `.env.sample` to `.env` and edit the environment variables to what you want.
- Run `docker-compose -f backend-service up -d` to start all the services needed by running the backend.
- Run `docker-compose up -d` to start the main backend container.

## Trending formula

$$
\frac{\ln{(post\_ len)}+ like\_ num/10+collect\_ num/8}{((now\_ in\_ hours-create\_ time\_ in\_ hours)/2+(now\_ in\_ hours-last\_ update\_ in\_ hours)/2+2)^{1.2}+10}
$$

## Tests

```bash
cargo tarpaulin --all-features --no-fail-fast --skip-clean  --verbose --timeout 300 --out Xml --exclude-files "src/bin/*" --follow-exec
```
