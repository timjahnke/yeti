# Yeti 
> The cooler, smarter cousin of Sass --watch (Sasquatch)

#### Compared to Dart Sass's watch, Yeti:
- Is minimal configuration. Doesn't need several CLI args and flags to invoke. 
- Is more efficient at file watching. Ignores file change events from creation, rename and metadata.
- Can triggers hot reloads of Sass files after build.
- Supports concurrency and collaboration. More than 1 person can work on a project and receive hot reloads.
- Only creates one file watcher and shares it among connections. 
- Is lightweight. Doesn't rely on the Node.js runtime.
- Uses [Grass](https://github.com/connorskees/grass) for 50% faster Sass compilation than Dart Sass. Default Sass compiler is Dart Sass.  

<br>
Yeti is a Sass build tool written in Rust for smarter file watching and hot reloading of Sass for Server Languages & Environments. At this time, Yeti is only available for Linux.   

## Prerequisites
- [Install](https://sass-lang.com/install/) Dart Sass CLI and [add it to your PATH](https://katiek2.github.io/path-doc/).

## Instructions
- Download the latest release, extract it and [add it to your PATH](https://katiek2.github.io/path-doc/).
  > You may find it easier to use `wget https://github.com/timjahnke/yeti/releases/download/v0.5.2-beta/yeti-v0.5.2-beta.tar.gz` in your PATH, extract and remove the tarball.
  
- In the project directory, create an empty `yeti.json` file.
- Open terminal in the project directory and run `yeti`.

  > Yeti will populate the empty JSON with the supported key-value pairs.
  > To opt in to using Grass, set `experimental` to `true`. 

- Update the `yeti.json` to meet your project structure.
- Add some code to import the websocket connection script from the server's client endpoint. (Examples below)
- Run `yeti` again and it launch the server.
- Open your project via a web browser. 
- If configured correctly, on save of Sass files, the page styles will be hot reloaded.


> Files events are no longer watched when the client disconnects or the terminal session is cancelled. (E.g. Ctrl+C)


### Why Rust?
As more build tool crates emerge, more build tools are being written in or converted to rust. It is with these hopes that Yeti may implement some of these features in the future. 
#### Crates
- [OXC](https://github.com/oxc-project/oxc)
- [Grass](https://github.com/connorskees/grass)
- [SWC](https://github.com/swc-project/swc)

#### Build Tools
- [Rolldown](https://github.com/rolldown/rolldown)
- [RsPack](https://github.com/web-infra-dev/rspack)
- [RsBuild](https://github.com/web-infra-dev/rsbuild)


### Why Yeti?
Yeti is intended to be a smarter, customizable version of the watch feature of Dart Sass. It's a play on words for being the cooler, smarter cousin of `sass --watch` (Sasquatch). It aims to be a hot reloading build tool as an executable binary like Dart Sass. The idea is to be a performant, collaborative and concurrent build tool. Yeti uses a single file watcher and a HTTP Websocket server for serving a dynamic JavaScript file, handling concurrent connections & tasks and to perform hot reloading after Sass builds. 

### Inspiration
Yeti is heavily inspired by [Vite](https://github.com/vitejs/vite) and [Dart Sass](https://github.com/sass/dart-sass). NPM package execution however relies on the Node.js runtime which can consume excess system resources such as RAM. In resource constrained environments (Remote Servers, Embedded Systems), it can be inefficient to build with Node Packages and execution can't be optimized due to Node being single-threaded. Rust introduces fast, memory-safe concurrency, a stricter compiler to prevent bugs/ memory leaks and also has very useful build-tool related crates. 


#### Examples for listening to the server
> Make sure to checks the environment to only add a listener in development

##### JavaScript
```
if(env === 'development') {
  fetch('yeti.json')
    .then(response => response.json())
    .then(data => {
      renderScriptTag(data.port);
    })
    .catch(error => {
      console.error('Error loading JSON file', error);
    });
  
  function renderScriptTag(port) {
    var scriptElement = document.createElement('script');
    scriptElement.src = `http://localhost:${port}/client`;
    document.head.appendChild(scriptElement);
  }
}

```
##### WordPress
```
if(getenv('WP_ENV') === 'development') {
  $jsonData = file_get_contents(ABSPATH . '/path/to/yeti.json');
  $data = json_decode($jsonData, true);
  $port = $data['port']; 
  wp_enqueue_script('websocket-listener', "http://localhost:{$port}/client", [], null);
}
```

