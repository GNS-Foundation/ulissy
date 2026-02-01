#!/usr/bin/env python3
"""
ULissy Post-Generation Fix Script v0.1.0
Patches known codegen bugs in the generated main.rs

Run after: ./compiler/ulissy/target/release/ulissy run examples/trip-protocol/src/trip_tests.ul
Usage: python3 fix_generated.py

Fixes:
  1. Test counters (0/0) - assignment expressions lost → inject unsafe increments
  2. isValidHandle - 'clean' variable empty → use name.replace("@","")
  3. TIT sequential key - wrong expected hash → correct to 630dcd...
  4. Breadcrumb signature verify - to_bytes() includes sig → use signing_bytes()
"""

import re
import sys
import os

MAIN_RS = "target/ulissy/run/src/main.rs"

def read_file(path):
    with open(path, 'r') as f:
        return f.read()

def write_file(path, content):
    with open(path, 'w') as f:
        f.write(content)

def apply_fixes(code):
    fixes_applied = []

    # =========================================================================
    # FIX 1: Test counter increments
    # The test() function has `let _ = Option::<()>::None;` where counter
    # increments should be. This is because the codegen can't handle
    # assignment expressions (testsRun = testsRun + 1).
    # =========================================================================
    old_test = '''async fn test(name: String, passed: bool) -> gns_runtime::RuntimeResult<()> {
    let _ = Option::<()>::None;
    if passed {
        let _ = Option::<()>::None;
        println!("{:?}", format!("✓ {}", name));
    } else {
        let _ = Option::<()>::None;
        println!("{:?}", format!("✗ {} FAILED", name));
    }'''

    new_test = '''async fn test(name: String, passed: bool) -> gns_runtime::RuntimeResult<()> {
    unsafe { testsRun += 1; }
    if passed {
        unsafe { testsPassed += 1; }
        println!("{:?}", format!("✓ {}", name));
    } else {
        unsafe { testsFailed += 1; }
        println!("{:?}", format!("✗ {} FAILED", name));
    }'''

    if old_test in code:
        code = code.replace(old_test, new_test)
        fixes_applied.append("FIX 1: Test counter increments restored")
    else:
        fixes_applied.append("FIX 1: SKIPPED (pattern not found - may already be fixed)")

    # =========================================================================
    # FIX 2: isValidHandle - clean variable
    # The codegen generates `let clean: String = String::new()` because the
    # original ULissy expression (name.replace("@","") or similar) generates
    # None through an unhandled method call pattern.
    # =========================================================================
    old_valid = 'let clean: String = String::new();'
    new_valid = 'let clean: String = name.replace("@", "");'

    if old_valid in code:
        # Only replace the one inside isValidHandle
        code = code.replace(old_valid, new_valid, 1)
        fixes_applied.append("FIX 2: isValidHandle clean = name.replace(\"@\",\"\")")
    else:
        fixes_applied.append("FIX 2: SKIPPED (pattern not found)")

    # =========================================================================
    # FIX 3: TIT sequential key expected hash
    # The expected hash in the ULissy test source is wrong.
    # SHA-256([0,1,2,...,31])[0:16] = 630dcd2966c4336691125448bbb25b4f
    # NOT bb63cf9f8f72d4c9e7f0be8ff64bc259
    # =========================================================================
    old_seq = 'bb63cf9f8f72d4c9e7f0be8ff64bc259'
    new_seq = '630dcd2966c4336691125448bbb25b4f'

    if old_seq in code:
        code = code.replace(old_seq, new_seq)
        fixes_applied.append("FIX 3: TIT sequential key hash corrected")
    else:
        fixes_applied.append("FIX 3: SKIPPED (hash not found)")

    # =========================================================================
    # FIX 4: Breadcrumb signature verification
    # bc.signature.verify(&bc.to_bytes(), ...) fails because to_bytes()
    # includes the signature field. The sign() method signed the pre-signature
    # bytes. Use signing_bytes() which excludes the signature field.
    # =========================================================================
    old_verify = 'bc.signature.verify(&bc.to_bytes(), &me.public_key)'
    new_verify = 'bc.signature.verify(&bc.signing_bytes(), &me.public_key)'

    count = code.count(old_verify)
    if count > 0:
        code = code.replace(old_verify, new_verify)
        fixes_applied.append(f"FIX 4: Breadcrumb verify → signing_bytes() ({count} occurrences)")
    else:
        fixes_applied.append("FIX 4: SKIPPED (verify pattern not found)")

    # =========================================================================
    # FIX 5: println with {:?} wraps strings in quotes
    # The test output shows "✓ alice" with quotes. This is cosmetic but
    # let's fix it for clean output where it matters most.
    # =========================================================================
    # This is cosmetic - skip for now

    return code, fixes_applied


def main():
    if not os.path.exists(MAIN_RS):
        print(f"ERROR: {MAIN_RS} not found.")
        print("Run the ULissy compiler first:")
        print("  ./compiler/ulissy/target/release/ulissy run examples/trip-protocol/src/trip_tests.ul")
        sys.exit(1)

    print("ULissy Post-Generation Fix Script v0.1.0")
    print(f"Patching: {MAIN_RS}")
    print()

    code = read_file(MAIN_RS)
    original_len = len(code)

    code, fixes = apply_fixes(code)

    for fix in fixes:
        status = "✓" if "SKIPPED" not in fix else "○"
        print(f"  {status} {fix}")

    write_file(MAIN_RS, code)
    print()
    print(f"Done. {original_len} → {len(code)} bytes")
    print()
    print("Now rebuild and run:")
    print("  cd target/ulissy/run && cargo build && cd ../../..")
    print("  ./target/ulissy/run/target/debug/trip_tests")


if __name__ == "__main__":
    main()
