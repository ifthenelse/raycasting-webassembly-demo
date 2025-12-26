#!/usr/bin/env node
// Check if port is in use and start the dev server (Vite) only if free.
// Usage: node tools/check_server_and_start.js

const net = require('net');
const { spawn } = require('child_process');
const PORT = process.env.PORT ? Number(process.env.PORT) : 5173;
const HOST = '127.0.0.1';

function checkPort(port, host, cb) {
  const socket = new net.Socket();
  let called = false;
  socket.setTimeout(1000);
  socket.on('connect', () => {
    called = true;
    socket.destroy();
    cb(true);
  });
  socket.on('timeout', () => {
    if (!called) {
      called = true;
      socket.destroy();
      cb(false);
    }
  });
  socket.on('error', () => {
    if (!called) {
      called = true;
      cb(false);
    }
  });
  socket.connect(port, host);
}

checkPort(PORT, HOST, (inUse) => {
  if (inUse) {
    console.log(`Dev port ${PORT} appears to be in use. Not starting another server.`);
    process.exit(0);
  } else {
    console.log(`Port ${PORT} is free — starting dev server.`);
    const child = spawn('npm', ['run', 'dev:internal'], { stdio: 'inherit', shell: true });
    child.on('exit', (code) => process.exit(code));
  }
});
