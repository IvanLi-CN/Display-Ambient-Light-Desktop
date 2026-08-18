import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

const manifestPath = "src-tauri/Cargo.toml";
const rootManifest = readFileSync(manifestPath, "utf8");
const vendorManifestPath = "src-tauri/vendor/ddc-hi/Cargo.toml";
const vendorManifest = readFileSync(vendorManifestPath, "utf8");

assert.match(
  rootManifest,
  /^image = \{ version = "0\.24", default-features = false, features = \["jpeg", "png"\] \}$/m,
);
assert.match(rootManifest, /^\[patch\.crates-io\]\nddc-hi = \{ path = "vendor\/ddc-hi" \}$/m);
assert.match(vendorManifest, /^name = "ddc-hi"$/m);
assert.match(vendorManifest, /^version = "0\.4\.1"$/m);
assert.match(vendorManifest, /^\[dependencies\.ddc\]\nversion = "\^0\.2\.0"$/m);
assert.match(vendorManifest, /^\[dependencies\.mccs\]\nversion = "\^0\.2"$/m);
assert.match(vendorManifest, /^\[dependencies\.mccs-caps\]\nversion = "\^0\.2"$/m);
assert.match(vendorManifest, /^\[dependencies\.mccs-db\]\nversion = "\^0\.2"$/m);
assert.ok(
  vendorManifest.includes(String.raw`[target."cfg(target_os = \"linux\")".dependencies.ddc-i2c]
version = "^0.2.1"
features = ["with-linux", "with-linux-enumerate"]
optional = true`),
  "linux ddc-i2c constraint must remain unchanged",
);
assert.ok(
  vendorManifest.includes(String.raw`[target."cfg(target_os = \"macos\")".dependencies.ddc-macos]
version = "^0.2.0"
optional = true`),
  "macOS ddc-macos constraint must remain unchanged",
);
assert.ok(
  vendorManifest.includes(String.raw`[target."cfg(windows)".dependencies.ddc-winapi]
version = "^0.2.0"
optional = true`),
  "Windows ddc-winapi constraint must remain unchanged",
);
assert.ok(
  vendorManifest.includes(String.raw`[target."cfg(windows)".dependencies.ddc-i2c]
version = "^0.2.1"
optional = true`),
  "Windows ddc-i2c constraint must remain unchanged",
);
assert.ok(
  vendorManifest.includes(String.raw`[target."cfg(windows)".dependencies.nvapi]
version = "^0.1.2"
features = ["i2c"]
optional = true
default-features = false`),
  "Windows nvapi constraint must remain unchanged",
);
assert.doesNotMatch(vendorManifest, /\b(?:git|registry)\s*=/);
assert.equal(existsSync("src-tauri/.cargo/audit.toml"), false, "audit ignores must not be checked in");

const metadata = JSON.parse(
  execFileSync(
    "cargo",
    ["metadata", "--manifest-path", manifestPath, "--locked", "--format-version", "1"],
    { encoding: "utf8", maxBuffer: 8 * 1024 * 1024 },
  ),
);
const ddcHi = metadata.packages.find(
  (pkg) => pkg.name === "ddc-hi" && pkg.version === "0.4.1",
);

assert.ok(ddcHi, "ddc-hi 0.4.1 must resolve");
assert.equal(ddcHi.source, null, "ddc-hi must resolve from the local patch");
assert.equal(ddcHi.manifest_path, resolve(vendorManifestPath));

for (const packageName of ["mccs", "mccs-caps", "mccs-db"]) {
  assert.ok(
    metadata.packages.some(
      (pkg) => pkg.name === packageName && pkg.version.startsWith("0.2."),
    ),
    `${packageName} 0.2 must resolve`,
  );
}

assert.ok(
  metadata.packages.some((pkg) => pkg.name === "ddc" && pkg.version.startsWith("0.2.")),
  "ddc 0.2 must remain resolved",
);
const localPackageNames = new Set(["ambient-light-control", "ddc-hi"]);
const cratesIoSource = "registry+https://github.com/rust-lang/crates.io-index";
assert.ok(
  metadata.packages.every(
    (pkg) =>
      pkg.source === cratesIoSource || (pkg.source === null && localPackageNames.has(pkg.name)),
  ),
  "resolved dependencies must use only crates.io or approved local packages",
);

console.log("Rust dependency security contract passed");
