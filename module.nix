packages:

{ lib
, pkgs
, config
, ... }:

let
  inherit (lib)
    mkEnableOption
    mkDefault
    mkIf
    mkOption
    optionalAttrs
    optional
    mkPackageOption;
  inherit (lib.types)
    bool
    path
    str
    submodule
    number
    array
    listOf;
  dataPath = "/var/lib/countr";
  cfg = config.services.countr;
in {
  options.services.countr = {
    enable = mkEnableOption "CountR";

    package = mkPackageOption packages.${pkgs.stdenv.hostPlatform.system} "default" { };

    user = mkOption {
      type = str;
      default = "countr";
      description = "User under which the service runs.";
    };

    group = mkOption {
      type = str;
      default = "countr";
      description = "Group under which the service runs.";
    };

    port = mkOption {
      type = number;
      default = 60887;
      description = "The port that the service is hosted on.";
    };
  };

  config = mkIf cfg.enable {
    systemd.services.countr = {
      description = "CountR";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];
      restartTriggers = [
        cfg.package
        cfg.port
      ];
      environment = {
        DATA_PATH = dataPath;
        POSTGRES_DB = "postgres://${cfg.user}@/countr";
        PORT = toString cfg.port;
      };

      serviceConfig = {
        Type = "simple";
        User = cfg.user;
        Group = cfg.group;
        StateDirectory = "countr";
        ExecStart = "${cfg.package}/bin/countr";
        Restart = "always";
      };
    };

    services.postgresql = {
      enable = mkDefault true;
      ensureDatabases = [ "countr" ];
      ensureUsers = [
        {
          name = "countr";
          ensureDBOwnership = true;
        }
      ];
    };

    users.users = optionalAttrs (cfg.user == "countr") {
      countr = {
        isSystemUser = true;
        group = cfg.group;
      };
    };

    users.groups = optionalAttrs (cfg.group == "countr") {
      countr = { };
    };
  };
}
