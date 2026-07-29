import sys
import argparse
import warnings
import os

# Suppress warnings from TensorFlow/PyTorch
os.environ["TF_CPP_MIN_LOG_LEVEL"] = "3"
warnings.filterwarnings("ignore")

def main():
    parser = argparse.ArgumentParser(description="OCR Chemical Structure to SMILES")
    parser.add_argument("image_path", help="Path to the chemical image")
    args = parser.parse_args()

    try:
        from DECIMER import predict_SMILES
    except ImportError:
        print("ERROR_NOT_INSTALLED", file=sys.stderr)
        sys.exit(1)

    try:
        smiles = predict_SMILES(args.image_path)
        print(smiles)
    except Exception as e:
        print(f"ERROR: {str(e)}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
