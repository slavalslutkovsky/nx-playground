export interface CIProvider {
  id: string;
  name: string;
  icon: string;
  configFile: string;
  features: string[];
}

export const providers: CIProvider[] = [
  {
    id: 'github-actions',
    name: 'GitHub Actions',
    icon: '🐙',
    configFile: '.github/workflows/ci.yml',
    features: ['matrix', 'caching', 'artifacts', 'reusable-workflows'],
  },
  {
    id: 'tekton',
    name: 'Tekton',
    icon: '🔧',
    configFile: '.tekton/pipeline.yaml',
    features: ['tasks', 'pipelines', 'triggers', 'kubernetes-native'],
  },
  {
    id: 'gitlab-ci',
    name: 'GitLab CI',
    icon: '🦊',
    configFile: '.gitlab-ci.yml',
    features: ['stages', 'caching', 'artifacts', 'includes'],
  },
  {
    id: 'circleci',
    name: 'CircleCI',
    icon: '⭕',
    configFile: '.circleci/config.yml',
    features: ['orbs', 'caching', 'workflows', 'parallelism'],
  },
];
