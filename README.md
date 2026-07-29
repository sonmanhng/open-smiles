# OpenSMILES

OpenSMILES is a desktop application built with Rust (`egui`/`eframe`) that extracts SMILES (Simplified Molecular-Input Line-Entry System) strings from chemical structure images. It utilizes a Python backend with the `DECIMER` library for Optical Chemical Structure Recognition (OCSR).

## Project Structure

- `src/` - Rust source code (GUI built with `egui` and `eframe`).
- `ocr_backend.py` - Python backend script using `DECIMER` to process images and predict SMILES.
- `Cargo.toml` & `Cargo.lock` - Rust package dependencies.

## Prerequisites

1. **Python 3.x**: Required for the OCSR backend.
2. **Rust**: Required for building the desktop application (`cargo`).

## Setup

### 1. Python Environment

It is recommended to set up a virtual environment:

```bash
python3 -m venv .venv
source .venv/bin/activate
# Install the required Python packages (e.g., DECIMER)
pip install git+https://github.com/Kohulan/DECIMER-Image_to_SMILES.git
```

### 2. Running the Application

To run the desktop application, use Cargo:

```bash
cargo run
```

## How it Works

The application provides a user interface to select an image of a chemical structure. It then executes the `ocr_backend.py` script, which uses the `DECIMER` deep learning model to translate the image into a SMILES string, which is then displayed in the UI.
