import {
  addPath,
  getBooleanInput,
  getInput,
  info,
  setFailed,
  setOutput,
  setSecret,
} from "@actions/core";
import { exec as execCommand } from "@actions/exec";
import { downloadTool } from "@actions/tool-cache";
import { createHash } from "node:crypto";
import { chmod, copyFile, mkdir, readFile, rm } from "node:fs/promises";
import { tmpdir, arch, platform } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repository = "jezdez/conda-express";
const releaseWorkflow = `${repository}/.github/workflows/release.yml`;
const releaseTagPattern = /^[0-9]+[.][0-9]+[.][0-9]+([.]post[0-9]+)?$/;

/**
 * Install a verified cx binary for the current runner and optionally bootstrap it.
 */
async function main() {
  const options = readOptions();
  const asset = platformAsset();
  const resolvedVersion = await resolveVersion(options.version, options.githubToken);
  const installDir = options.installDir || path.join(runnerTemp(), "cx", "bin");
  const workDir = path.join(runnerTemp(), `cx-download-${resolvedVersion}`);

  await rm(workDir, { recursive: true, force: true });
  await mkdir(workDir, { recursive: true });
  await mkdir(installDir, { recursive: true });

  const baseUrl = `https://github.com/${repository}/releases/download/${resolvedVersion}`;
  const assetPath = path.join(workDir, asset.name);
  const checksumName = `${asset.name.replace(/[.]exe$/, "")}.sha256`;
  const checksumPath = path.join(workDir, checksumName);

  info(`Downloading ${asset.name} from conda-express ${resolvedVersion}`);
  await downloadFile(`${baseUrl}/${asset.name}`, assetPath);
  await downloadFile(`${baseUrl}/${checksumName}`, checksumPath);
  await verifyChecksum(assetPath, checksumPath, asset.name);

  if (options.verifyAttestation) {
    await verifyAttestation(assetPath, resolvedVersion, options.githubToken);
  }

  const binaryPath = path.join(installDir, asset.binaryName);
  await copyFile(assetPath, binaryPath);
  await chmod(binaryPath, 0o755);

  if (options.addToPath) {
    addPath(installDir);
  }

  setOutput("asset-name", asset.name);
  setOutput("cx-path", binaryPath);
  setOutput("install-dir", installDir);
  setOutput("version", resolvedVersion);

  if (options.bootstrap) {
    await bootstrap(binaryPath);
  }
}

/**
 * Read action inputs and mask the token when running inside GitHub Actions.
 */
function readOptions() {
  const githubToken = getInput("github-token");
  if (githubToken && process.env.GITHUB_ACTIONS === "true") {
    setSecret(githubToken);
  }

  return {
    addToPath: getBooleanInput("add-to-path"),
    bootstrap: getBooleanInput("bootstrap"),
    githubToken,
    installDir: getInput("install-dir"),
    verifyAttestation: getBooleanInput("verify-attestation"),
    version: stripLeadingV(getInput("version")),
  };
}

function stripLeadingV(value) {
  return value.trim().replace(/^v/, "");
}

function runnerTemp() {
  return process.env.RUNNER_TEMP || tmpdir();
}

/**
 * Resolve the release asset name for the current runner platform.
 */
function platformAsset() {
  const os = platform();
  const cpu = arch();

  if (os === "linux" && cpu === "x64") {
    return { name: "cx-x86_64-unknown-linux-gnu", binaryName: "cx" };
  }
  if (os === "linux" && cpu === "arm64") {
    return { name: "cx-aarch64-unknown-linux-gnu", binaryName: "cx" };
  }
  if (os === "darwin" && cpu === "x64") {
    return { name: "cx-x86_64-apple-darwin", binaryName: "cx" };
  }
  if (os === "darwin" && cpu === "arm64") {
    return { name: "cx-aarch64-apple-darwin", binaryName: "cx" };
  }
  if (os === "win32" && cpu === "x64") {
    return { name: "cx-x86_64-pc-windows-msvc.exe", binaryName: "cx.exe" };
  }

  throw new Error(`Unsupported runner platform: ${os}/${cpu}`);
}

/**
 * Pick the cx release version to install.
 *
 * Explicit `version` wins. Otherwise, the action tries to infer the version
 * from the action ref before falling back to GitHub's latest release API.
 */
async function resolveVersion(requestedVersion, githubToken) {
  if (requestedVersion) {
    return requestedVersion;
  }

  const actionRef = stripLeadingV(process.env.GITHUB_ACTION_REF || "");
  if (releaseTagPattern.test(actionRef)) {
    return actionRef;
  }

  const inferredActionRef = inferActionRefFromPath();
  if (releaseTagPattern.test(inferredActionRef)) {
    return inferredActionRef;
  }

  const release = await fetchJson(
    `https://api.github.com/repos/${repository}/releases/latest`,
    githubToken,
  );
  const tagName = stripLeadingV(String(release.tag_name || ""));
  if (!releaseTagPattern.test(tagName)) {
    throw new Error(`Latest release tag is not a conda-express version: ${tagName}`);
  }
  return tagName;
}

/**
 * Infer the action ref from GitHub's checked-out action path.
 *
 * JavaScript actions do not receive `github.action_ref` as an automatic input,
 * so this keeps the common `uses: ...@26.5.2` case aligned with the matching
 * release asset without requiring callers to repeat `version`.
 */
function inferActionRefFromPath() {
  const actionPath = fileURLToPath(import.meta.url);
  const segments = actionPath.split(path.sep);
  const actionRootIndex = segments.lastIndexOf(".github");
  if (actionRootIndex < 1) {
    return "";
  }
  return stripLeadingV(segments[actionRootIndex - 1] || "");
}

async function fetchJson(url, githubToken) {
  const response = await fetch(url, {
    headers: requestHeaders(githubToken, { accept: "application/vnd.github+json" }),
  });
  if (!response.ok) {
    throw new Error(`GET ${url} failed with HTTP ${response.status}`);
  }
  return response.json();
}

/**
 * Download a release asset through the official Actions tool-cache helper.
 */
async function downloadFile(url, destination) {
  const downloadedPath = await downloadTool(url);
  await copyFile(downloadedPath, destination);
  await chmod(destination, 0o600);
}

function requestHeaders(githubToken, extra = {}) {
  const headers = {
    "user-agent": "setup-cx-action",
    ...extra,
  };
  if (githubToken) {
    headers.authorization = `Bearer ${githubToken}`;
  }
  return headers;
}

/**
 * Verify the downloaded binary against the release checksum file.
 */
async function verifyChecksum(assetPath, checksumPath, assetName) {
  const checksumText = await readFile(checksumPath, "utf8");
  const checksum = checksumText
    .split(/\r?\n/)
    .map((line) => line.trim().split(/\s+/))
    .find((parts) => parts.length >= 2 && parts[1] === assetName);

  if (!checksum) {
    throw new Error(`No checksum entry found for ${assetName}`);
  }

  const expected = checksum[0].toLowerCase();
  const actual = await sha256(assetPath);
  if (actual !== expected) {
    throw new Error(`Checksum mismatch for ${assetName}: expected ${expected}, got ${actual}`);
  }
  info(`${assetName}: checksum verified`);
}

async function sha256(filePath) {
  const data = await readFile(filePath);
  return createHash("sha256").update(data).digest("hex");
}

/**
 * Verify the binary's GitHub Artifact Attestation.
 *
 * The GitHub REST API can fetch attestation bundles, but the security-sensitive
 * signature and policy verification is delegated to `gh attestation verify`.
 */
async function verifyAttestation(assetPath, version, githubToken) {
  if (!githubToken) {
    throw new Error("github-token is required when verify-attestation is true");
  }

  await run("gh", [
    "attestation",
    "verify",
    assetPath,
    "--repo",
    repository,
    "--signer-workflow",
    releaseWorkflow,
    "--source-ref",
    `refs/tags/${version}`,
    "--deny-self-hosted-runners",
  ], {
    env: { ...process.env, GH_TOKEN: githubToken },
    quiet: true,
  });
  info(`${path.basename(assetPath)}: attestation verified`);
}

/**
 * Trigger automatic bootstrap through a regular conda command.
 */
async function bootstrap(cxPath) {
  await run(cxPath, ["info", "--json"]);
}

/**
 * Execute a command with arguments and surface captured output on failure.
 */
async function run(command, args, options = {}) {
  let stdout = "";
  let stderr = "";
  const exitCode = await execCommand(command, args, {
    env: options.env || process.env,
    ignoreReturnCode: true,
    silent: options.quiet === true,
    listeners: {
      stdout: (data) => {
        stdout += data.toString();
      },
      stderr: (data) => {
        stderr += data.toString();
      },
    },
  });

  if (exitCode !== 0) {
    const details = [stderr, stdout].filter(Boolean).join("\n").trim();
    throw new Error(details || `${command} failed with exit code ${exitCode}`);
  }
}

try {
  await main();
} catch (error) {
  setFailed(error instanceof Error ? error.message : String(error));
}
