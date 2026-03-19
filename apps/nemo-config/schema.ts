export const CONSUL_PREFIX = 'nemo';

export const ServiceConfigKeys = {
  serviceUrl: (serviceId: string) => `${CONSUL_PREFIX}.${serviceId}.url`,
  serviceMetadata: (serviceId: string) => `${CONSUL_PREFIX}.metadata.${serviceId}`,
} as const;

export const ConsulKeys = {
  serviceUrl: (serviceId: string) => `${CONSUL_PREFIX}/${serviceId}/url`,
  serviceMetadata: (serviceId: string) => `${CONSUL_PREFIX}/metadata/${serviceId}`,
  servicePrefix: (serviceId: string) => `${CONSUL_PREFIX}/${serviceId}/`,
  metadataPrefix: `${CONSUL_PREFIX}/metadata/`,
} as const;

export const API_CONFIG_KEYS = {
  allServices: `${CONSUL_PREFIX}/`,
} as const;
