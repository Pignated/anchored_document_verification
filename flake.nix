{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    my-repo.url = "github:pignated/flakes";
  };

  outputs = { self, nixpkgs, my-repo  }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      myPackage = pkgs.callPackage (my-repo + "/rust/default.nix") { };
      STATE_FILE = "anvil.json";
      PID_FILE = ".anvil.pid";
      LOG_FILE = "anvil.log";
      start-anvil = pkgs.writeScriptBin "start-anvil" ''
        #!/usr/bin/env bash
        echo "Starting anvil..."
        if [ -f "${PID_FILE}" ] && kill -0 $(cat ${PID_FILE}) 2>/dev/null; then
          echo "anvil already running (PID: $(cat ${PID_FILE}))."
          exit 1
        fi
        FLAGS="--dump-state ${STATE_FILE}"
        if [ -f "${STATE_FILE}" ]; then
          echo "starting anvil from existing state file"
          FLAGS="--load-state=${STATE_FILE} $FLAGS"
        else
          echo "no existing state found, creating at $STATE_FILE"
        fi
        anvil $FLAGS > ${LOG_FILE} 2>&1 &
        ANVIL_PID=$!
        echo $ANVIL_PID > ${PID_FILE}
        echo "Anvil active (PID: $ANVIL_PID)"
      '';
      stop-anvil = pkgs.writeScriptBin "stop-anvil" ''
        #!/usr/bin/env bash
        if [ ! -f "${PID_FILE}" ]; then
          echo "No active Anvil process"
          exit 1
        fi
        PID=$(cat ${PID_FILE})
        if kill -0 $PID 2>/dev/null; then
          echo "Stopping Anvil (PID: $PID) gracefully to save state."
          kill -SIGINT $PID
          while kill -0 $PID 2>/dev/null; do sleep 0.5; done
          echo "State saved successfully to ${STATE_FILE}"
        else
          echo "Proces was alreday dead, cleanup triggered"
        fi
        rm -f ${PID_FILE}
      '';

    in
    {
      packages.x86_64-linux.default = myPackage;

      devShells.x86_64-linux.default = pkgs.mkShell {
        inputsFrom = [ myPackage ];
        buildInputs = [ 
          pkgs.foundry 
          start-anvil
          stop-anvil
        ];
        shellHook = myPackage.shellHook or "";
      };
    };
}
