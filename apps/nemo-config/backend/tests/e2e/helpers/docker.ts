import { exec } from 'child_process';
import { promisify } from 'util';
import { CONFIG } from '../config';

const execAsync = promisify(exec);

/**
 * Executes a shell command and returns the result
 */
export async function runCommand(cmd: string): Promise<{ stdout: string; stderr: string; exitCode: number }> {
  try {
    const result = await execAsync(cmd);
    return { stdout: result.stdout.toString(), stderr: result.stderr.toString(), exitCode: 0 };
  } catch (error: any) {
    return { stdout: error.stdout?.toString() || '', stderr: error.stderr?.toString() || '', exitCode: 1 };
  }
}

/**
 * Checks if a container exists on a host
 */
export async function containerExists(containerName: string, host: string = 'localhost'): Promise<boolean> {
  const cmd = host === 'localhost' 
    ? `docker ps -a --filter name=${containerName} --format '{{.Names}}'`
    : `ssh ${host} "docker ps -a --filter name=${containerName} --format '{{.Names}}'"`;
    
  const result = await runCommand(cmd);
  return result.stdout.trim() === containerName;
}

/**
 * Checks if a container is running on a host
 */
export async function containerIsRunning(containerName: string, host: string = 'localhost'): Promise<boolean> {
  const cmd = host === 'localhost'
    ? `docker ps --filter name=${containerName} --filter status=running --format '{{.Names}}'`
    : `ssh ${host} "docker ps --filter name=${containerName} --filter status=running --format '{{.Names}}'\"`;
    
  const result = await runCommand(cmd);
  return result.stdout.trim() === containerName;
}

/**
 * Gets container logs from a host
 */
export async function getContainerLogs(containerName: string, host: string = 'localhost', tail: number = 100): Promise<string[]> {
  const cmd = host === 'localhost'
    ? `docker logs --tail ${tail} ${containerName} 2>&1`
    : `ssh ${host} "docker logs --tail ${tail} ${containerName} 2>&1\"`;
    
  const result = await runCommand(cmd);
  if (result.exitCode !== 0) {
    throw new Error(`Failed to get logs: ${result.stderr}`);
  }
  return result.stdout.split('\n').filter(line => line.trim() !== '');
}

/**
 * Waits for a container to reach a specific status
 */
export async function waitForContainerStatus(
  containerName: string, 
  expectedStatus: 'running' | 'stopped' | 'exited', 
  host: string = 'localhost',
  timeoutMs: number = CONFIG.CONTAINER_READY_TIMEOUT
): Promise<boolean> {
  const startTime = Date.now();
  
  while (Date.now() - startTime < timeoutMs) {
    let isRunning = false;
    try {
      isRunning = await containerIsRunning(containerName, host);
    } catch (error) {
      // Container might not exist yet
    }
    
    const isExited = !isRunning && await containerExists(containerName, host);
    
    if (
      (expectedStatus === 'running' && isRunning) ||
      (expectedStatus === 'stopped' && !isRunning && await containerExists(containerName, host)) ||
      (expectedStatus === 'exited' && isExited)
    ) {
      return true;
    }
    
    // Wait a bit before checking again
    await new Promise(resolve => setTimeout(resolve, 500));
  }
  
  return false;
}

/**
 * Stops a container on a host
 */
export async function stopContainer(containerName: string, host: string = 'localhost'): Promise<void> {
  const cmd = host === 'localhost'
    ? `docker stop ${containerName}`
    : `ssh ${host} "docker stop ${containerName}\"`;
    
  const result = await runCommand(cmd);
  if (result.exitCode !== 0) {
    throw new Error(`Failed to stop container: ${result.stderr}`);
  }
}

/**
 * Starts a container on a host
 */
export async function startContainer(containerName: string, host: string = 'localhost'): Promise<void> {
  const cmd = host === 'localhost'
    ? `docker start ${containerName}`
    : `ssh ${host} "docker start ${containerName}\"`;
    
  const result = await runCommand(cmd);
  if (result.exitCode !== 0) {
    throw new Error(`Failed to start container: ${result.stderr}`);
  }
}

/**
 * Removes a container and its directory on a host
 */
export async function removeContainer(containerName: string, serviceId: string, host: string = 'localhost'): Promise<void> {
  const expandedDir = `~/workspace/nemo/${serviceId}`.replace(/^~/, process.env.HOME || '');
  
  let cmd: string;
  if (host === 'localhost') {
    cmd = `docker rm -f ${containerName} 2>/dev/null || true && rm -rf ${expandedDir}`;
  } else {
    cmd = `ssh ${host} "docker rm -f ${containerName} 2>/dev/null || true && rm -rf ${expandedDir}\"`;
  }
  
  const result = await runCommand(cmd);
  if (result.exitCode !== 0) {
    throw new Error(`Failed to remove container: ${result.stderr}`);
  }
}