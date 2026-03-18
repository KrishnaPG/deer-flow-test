import { useState, useEffect } from 'react';

const DEPLOY_PATH_STORAGE_KEY = 'nemo-config-deploy-path';

export function useDeployPath() {
  const [deployPath, setDeployPath] = useState<string>(() => {
    if (typeof window !== 'undefined') {
      return localStorage.getItem(DEPLOY_PATH_STORAGE_KEY) || '~/workspace/nemo';
    }
    return '~/workspace/nemo';
  });

  useEffect(() => {
    localStorage.setItem(DEPLOY_PATH_STORAGE_KEY, deployPath);
  }, [deployPath]);

  return { deployPath, setDeployPath };
}