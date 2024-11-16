// This file is generated by `mise render-help`
// Do not edit this file directly

export type Command = {
  hide: boolean;
  subcommands?: {
    [key: string]: Command;
  };
};
export const commands: { [key: string]: Command } = {
  activate: {
    hide: false,
  },
  alias: {
    hide: false,
    subcommands: {
      get: {
        hide: false,
      },
      ls: {
        hide: false,
      },
      set: {
        hide: false,
      },
      unset: {
        hide: false,
      },
    },
  },
  asdf: {
    hide: true,
  },
  backends: {
    hide: false,
    subcommands: {
      ls: {
        hide: false,
      },
    },
  },
  "bin-paths": {
    hide: false,
  },
  cache: {
    hide: false,
    subcommands: {
      clear: {
        hide: false,
      },
      prune: {
        hide: false,
      },
    },
  },
  completion: {
    hide: false,
  },
  config: {
    hide: false,
    subcommands: {
      generate: {
        hide: false,
      },
      get: {
        hide: false,
      },
      ls: {
        hide: false,
      },
      set: {
        hide: false,
      },
    },
  },
  current: {
    hide: true,
  },
  deactivate: {
    hide: false,
  },
  direnv: {
    hide: false,
    subcommands: {
      envrc: {
        hide: true,
      },
      exec: {
        hide: true,
      },
      activate: {
        hide: false,
      },
    },
  },
  doctor: {
    hide: false,
  },
  env: {
    hide: false,
  },
  exec: {
    hide: false,
  },
  generate: {
    hide: false,
    subcommands: {
      "git-pre-commit": {
        hide: false,
      },
      "github-action": {
        hide: false,
      },
      "task-docs": {
        hide: false,
      },
    },
  },
  global: {
    hide: true,
  },
  "hook-env": {
    hide: true,
  },
  "hook-not-found": {
    hide: true,
  },
  implode: {
    hide: false,
  },
  install: {
    hide: false,
  },
  latest: {
    hide: false,
  },
  link: {
    hide: false,
  },
  local: {
    hide: true,
  },
  ls: {
    hide: false,
  },
  "ls-remote": {
    hide: false,
  },
  outdated: {
    hide: false,
  },
  plugins: {
    hide: false,
    subcommands: {
      install: {
        hide: false,
      },
      link: {
        hide: false,
      },
      ls: {
        hide: false,
      },
      "ls-remote": {
        hide: false,
      },
      uninstall: {
        hide: false,
      },
      update: {
        hide: false,
      },
    },
  },
  prune: {
    hide: false,
  },
  registry: {
    hide: false,
  },
  "render-help": {
    hide: true,
  },
  "render-mangen": {
    hide: true,
  },
  reshim: {
    hide: false,
  },
  run: {
    hide: false,
  },
  "self-update": {
    hide: false,
  },
  set: {
    hide: false,
  },
  settings: {
    hide: false,
    subcommands: {
      add: {
        hide: false,
      },
      get: {
        hide: false,
      },
      ls: {
        hide: false,
      },
      set: {
        hide: false,
      },
      unset: {
        hide: false,
      },
    },
  },
  shell: {
    hide: false,
  },
  sync: {
    hide: false,
    subcommands: {
      node: {
        hide: false,
      },
      python: {
        hide: false,
      },
    },
  },
  tasks: {
    hide: false,
    subcommands: {
      deps: {
        hide: false,
      },
      edit: {
        hide: false,
      },
      info: {
        hide: false,
      },
      ls: {
        hide: false,
      },
      run: {
        hide: false,
      },
    },
  },
  "test-tool": {
    hide: true,
  },
  trust: {
    hide: false,
  },
  uninstall: {
    hide: false,
  },
  unset: {
    hide: false,
  },
  upgrade: {
    hide: false,
  },
  usage: {
    hide: true,
  },
  use: {
    hide: false,
  },
  version: {
    hide: false,
  },
  watch: {
    hide: false,
  },
  where: {
    hide: false,
  },
  which: {
    hide: false,
  },
};
