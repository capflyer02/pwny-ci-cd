# 🌤️ Pwny Weather CI/CD App

A fully automated Rust + Axum web application that fetches real-time weather data from the **Weather Underground API** and deploys globally on **Fly.io**, using a **GitHub Actions CI/CD pipeline**.

---

## 🦀 Project Overview

**Stack**

| Component | Technology |
|------------|-------------|
| Web Framework | [Axum](https://github.com/tokio-rs/axum) |
| Language | Rust |
| API | Weather Underground (PWS API) |
| Containerization | Docker (multi-stage build) |
| CI/CD | GitHub Actions |
| Deployment | Fly.io |

The app automatically:
1. Builds and tests across multiple OSs (Linux, macOS, Windows)
2. Tags and publishes GitHub releases
3. Builds Docker images
4. Deploys automatically to Fly.io when a new version tag (`v*`) is pushed

---

## 🧱 Directory Structure

├── src/
│ └── main.rs
├── Cargo.toml
├── Dockerfile
├── fly.toml
├── .github/
│ └── workflows/
│ ├── ci.yml
│ ├── release.yml
│ └── deploy-fly.yml
└── docs/
├── pwny-weather-ci-cd-fly-guide-v3.docx
└── pwny-weather-ci-cd-fly-guide-v3.pdf



---

## 🌤️ Weather Underground API

The app uses the PWS endpoint:

GET https://api.weather.com/v2/pws/observations/current

?stationId=<STATION_ID>&format=json&units=e&apiKey=<WU_API_KEY>


Results are parsed with `serde` into Rust structs and returned as JSON via Axum.

---

## ☁️ Fly.io Deployment

**Fly.io** runs your container close to users worldwide with HTTPS, scaling, and logs built-in.

Deploy manually (once):

```bash
fly launch
fly secrets set WU_API_KEY=your_api_key
fly deploy


Then automatic deployments happen via GitHub Actions on tag push.

🔄 CI/CD Automation
Build and Test (ci.yml)

Runs on every push or pull request.

Release (release.yml)

Triggered by tags (v1.*), builds a release binary.

Deploy (deploy-fly.yml)

Triggered by the same tags, deploys to Fly.io automatically.

🧰 GitHub Actions Cheat Sheet
Purpose	Action
Checkout code	actions/checkout@v4
Setup Rust	dtolnay/rust-toolchain@stable
Cache dependencies	actions/cache@v4
Deploy to Fly.io	superfly/flyctl-actions/setup-flyctl@master


Triggering deployment:

git tag v1.4.0
git push origin v1.4.0



📄 Documentation

Comprehensive documentation:

pwny-weather-ci-cd-fly-guide-v3.pdf

pwny-weather-ci-cd-fly-guide-v3.docx

🔒 Security

Use GitHub Secrets for WU_API_KEY, TAG_TOKEN, and FLY_API_TOKEN.

Never commit credentials or tokens.

Restrict CORS origins for sensitive data.


📜 License

MIT License © 2025
