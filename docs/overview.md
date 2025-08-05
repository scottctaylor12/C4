# C4: Cross Compatible Command and Control

Let's learn a little more about the project and what is going on here.

## Problem

Different C2 frameworks offer different external C2 options.
Oftentimes, these external C2 options are implemented in a way that is highly specific to the framework using language-specific SDK's and message formats.
That said, if red team operators need to communicate over a specific site (AWS S3 for example) then they are limited to certain C2 frameworks. If the operator is developer-savy, they *could* re-invent the wheel and implement the external C2 in their framework of choice, but that's tedious...

## Solution

To ease the burden of implementing external C2 both team server and agent side, the C4 was developed!
C4 is a collection of plugins that handle sending and receiving messages over various trusted sites.
These "plugins" are compiled WebAssembly modules that can be loaded into numerous programming languages (hence, the "Cross Compatible" in the project name).

## WebAssembly (WASM)

The C4 plugins are written in Rust.
However, no need to fear, no Rust knowledge is required to use this project.
The plugins are compiled to a WebAssembly module (.wasm files) that can be loaded by numerous programming languages including but not limited to Go, Rust, and Python.
That said, to use the plugins, a WebAssembly runtime must be loaded into the C2 framework and/or agent in order to use these modules.

## Extism

Behind the scenes, the [Extism](https://extism.org/) project is what makes this all possible.
Extism is a cross-language framework for building and running WebAssembly.
Extism has [Plugin Development Kits (PDK)](https://extism.org/docs/concepts/pdk) that allow developers to write WASM modules in multiple languages. 
I chose to write the plugins in Rust to keep the compiled module sizes smaller.
Additionally, the Rust language is heavily supported in the WASM community.
Extism also maintains ["Host SDKs"](https://extism.org/docs/concepts/host-sdk) which allow users to take a compiled Extism plugin (.wasm file) and run it across multiple lanugages.
This is perfect for all you malware developers out there writing in various programming languages!

## Planting C4

The C4 project contains numerous External C2 modules that can be used in your software.
Read on to find your language(s) of choice and start planting some C4!