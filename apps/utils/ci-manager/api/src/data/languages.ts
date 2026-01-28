export interface Language {
  id: string;
  name: string;
  icon: string;
  buildTools: string[];
  testCommands: string[];
  lintCommands: string[];
}

export const languages: Language[] = [
  {
    id: 'rust',
    name: 'Rust',
    icon: '🦀',
    buildTools: ['cargo', 'cargo-nextest'],
    testCommands: ['cargo test', 'cargo nextest run'],
    lintCommands: ['cargo clippy', 'cargo fmt --check'],
  },
  {
    id: 'go',
    name: 'Go',
    icon: '🐹',
    buildTools: ['go'],
    testCommands: ['go test ./...'],
    lintCommands: ['golangci-lint run', 'go fmt ./...'],
  },
  {
    id: 'node',
    name: 'Node.js',
    icon: '🟢',
    buildTools: ['npm', 'yarn', 'pnpm'],
    testCommands: ['npm test', 'yarn test', 'pnpm test'],
    lintCommands: ['npm run lint', 'eslint .', 'biome check .'],
  },
  {
    id: 'bun',
    name: 'Bun',
    icon: '🥟',
    buildTools: ['bun'],
    testCommands: ['bun test'],
    lintCommands: ['bun run lint', 'biome check .'],
  },
  {
    id: 'python',
    name: 'Python',
    icon: '🐍',
    buildTools: ['pip', 'poetry', 'uv'],
    testCommands: ['pytest', 'python -m unittest'],
    lintCommands: ['ruff check .', 'black --check .', 'mypy .'],
  },
  {
    id: 'kotlin',
    name: 'Kotlin',
    icon: '🇰',
    buildTools: ['gradle', 'maven'],
    testCommands: ['./gradlew test', 'mvn test'],
    lintCommands: ['./gradlew ktlintCheck', 'ktlint'],
  },
];
