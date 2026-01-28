const API_BASE_URL = 'http://localhost:3001/api';

export interface Language {
  id: string;
  name: string;
  icon: string;
  buildTools: string[];
  testCommands: string[];
  lintCommands: string[];
}

export interface CIProvider {
  id: string;
  name: string;
  icon: string;
  configFile: string;
  features: string[];
}

export interface GeneratePipelineRequest {
  languages: string[];
  provider: string;
  options: {
    lint: boolean;
    test: boolean;
    build: boolean;
    cache: boolean;
  };
}

export interface GeneratePipelineResponse {
  configFile: string;
  yaml: string;
  languages: string[];
  provider: string;
}

export async function fetchLanguages(): Promise<Language[]> {
  const response = await fetch(`${API_BASE_URL}/languages`);
  if (!response.ok) {
    throw new Error('Failed to fetch languages');
  }
  return response.json();
}

export async function fetchProviders(): Promise<CIProvider[]> {
  const response = await fetch(`${API_BASE_URL}/providers`);
  if (!response.ok) {
    throw new Error('Failed to fetch providers');
  }
  return response.json();
}

export async function generatePipeline(
  request: GeneratePipelineRequest,
): Promise<GeneratePipelineResponse> {
  const response = await fetch(`${API_BASE_URL}/pipelines/generate`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(request),
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error || 'Failed to generate pipeline');
  }

  return response.json();
}
