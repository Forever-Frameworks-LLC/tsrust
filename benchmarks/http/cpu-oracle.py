"""Independent complex-number reference, outside every server and timed phase."""
import json
from pathlib import Path

def render(identifier):
    checksum = escaped = 0
    for y in range(64):
        for x in range(96):
            point = complex(-2.0 + x * 3.0 / 96.0 + (identifier % 7) * 0.0001,
                            -1.0 + y * 2.0 / 64.0 + (identifier % 11) * 0.0001)
            z = 0j
            iterations = 0
            while iterations < 256 and z.real * z.real + z.imag * z.imag <= 4.0:
                z = z * z + point
                iterations += 1
            checksum = (checksum + iterations * (y * 96 + x + 1)) % 1000000007
            escaped += iterations < 256
    return {"id": identifier, "checksum": checksum, "escaped": escaped}

if __name__ == "__main__":
    import sys
    output = Path(__file__).with_name("cpu-oracle.json")
    results = [render(i) for i in range(1, 129)]
    if "--check" in sys.argv:
        assert json.loads(output.read_text()) == results
    else:
        output.write_text(json.dumps(results, separators=(",", ":"), sort_keys=True) + "\n")
