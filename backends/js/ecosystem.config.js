module.exports = {
  apps: [
    {
      name: "access-protocol-server",
      script: "dist/index.js",
      watch: ".",
      instances: "max",
      exec_mode: "cluster",
    },
  ],
};
