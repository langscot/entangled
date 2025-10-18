# Entangled

This is my first attempt at using Rust to write my own protocol on top of TCP/IP to facilitate a client/server setup. Clients collect device and OS level metrics and report back to the server. Server writes metrics to a SQLite database and exposes them via a simple web interface. The web interface allows administrators to "push" metric collection configurations to clients which tells clients which metrics to collect and at which frequency to report back to the server.

The main motivation behind this is to learn more about Rust and TCP/IP/networking. My plan is once the whole software "works" I will go back with a focus on security to learn about those concepts next.

Current stage: Pinging & Ponging