# Lightron: Web Server
# Table Of Contents
- [About The Project](#about-the-project)
  * [Built With](#built-with)
- [Getting Started](#getting-started)
  * [Installation](#installation)
    + [Windows](#windows)
    + [Linux](#linux)
    + [Docker](#docker)
- [Usage](#usage)
- [Acknowledgements](#acknowledgements)

# About the Project
The Lightron Web Server is a lightweight web server available on two operating systems which are Windows and Linux. It is a web server developed using Rust. Rust has allowed the Lightron Web Server to be concurrent due to which a single thread can handle multiple requests allowing minimum load on the processor. You will be able to host your website using both HTTP and HTTPS. Both HTTP/1.1 and HTTP/2.0 version are supported. GUI support has been provided so that the user finds it easier to host the website. You can see the percentage of CPU resources utilized, percentage of RAM utilization by the web server in the GUI itself. Overall CPU & Memory utilization can also be seen in the statistics tab on top of the GUI. The main objective of creating this web server is to allow user to host their websites with ease with minimal load given on the machine's internal resources.

## Built With
The Lightron Web Server has been developed using the following languages and various frameworks:<br>
[Rust](https://www.rust-lang.org/)<br>
[Fltk](https://docs.rs/fltk/)<br>
[Tokio](https://tokio.rs/)<br>
Kudos, To all other rust based libraries/crates which have also played equal & important part in the development of this project.

# Getting Started
## Installation
<br>
To know how to install and get started with the Lightron Web Server follow these simple steps:<br>

### Windows
1. Download [Lightron_Setup_x86-64.exe](https://github.com/lightron/lightron/releases/download/v0.1.0/Lightron_Setup_x86-64.exe), double click it to launch the installer, and then follow the instructions.
2. You will be asked what directory to install .exe file in.
3. The installer will create an exe icon on the desktop and a menu-item under "All Programs"..<br>

### Linux
* Debian Based Distribution
    1. Download the .deb file by clicking [here](https://github.com/lightron/lightron/releases/download/v0.1.0/lightron-0.1.0.deb).
    2. Open Terminal and change the directory to where the debian package has been downloaded.
    3. Type the below command to install the package.
        ```
        sudo apt-get install ./lightron-0.1.0.deb
        ```
* Red Hat Based Distribution
    1. Download the .rpm file by clicking [here](https://github.com/lightron/lightron/releases/download/v0.1.0/lightron-0.1.0-1.fc33.x86_64.rpm).
    2. Open Terminal and change the directory to where the rpm package has been downloaded.
    4. RPM Fusion repository must be added to the system by following command, Ignore If already added.
       ```
       sudo dnf install https://download1.rpmfusion.org/free/fedora/rpmfusion-free-release-$(rpm -E %fedora).noarch.rpm
       ```
       ```
       sudo dnf install https://download1.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-$(rpm -E %fedora).noarch.rpm
       ```
    3. Type the below command to install the package.
        ```
        sudo dnf install ./lightron-0.1.0-1.fc33.x86_64.rpm
        ```
* Arch Based Distribution
    1. Download the .pkg.tar.zst file by clicking [here](https://github.com/lightron/lightron/releases/download/v0.1.0/lightron-0.1.0-1-x86_64.pkg.tar.zst).
    2. Open Terminal and change the directory to where the arch package has been downloaded.
    3. Type the below command to install the package.
        ```
        sudo pacman -U ./lightron-0.1.0-1-x86_64.pkg.tar.zst
        ```
### Docker
1. Download the [Dockerfile](https://github.com/lightron/lightron/blob/main/Dockerfile).
2. Open Terminal and change the directory to where the Dockerfile has been downloaded.
3. Build Docker Image.
    ```
    docker build -t="lightron" .
    ```
4. Run Docker Container.
    ```
    docker run -d -p 80:80 -v [volume name]:/var/www -v [path to lightron.conf file on host system]:/etc/lightron.conf lightron
    ```
    On Windows system volume can be found at \\\wsl$\docker-desktop-data\version-pack-data\community\docker\volumes<br>
    On Linux /var/lib/docker/volumes/

# Usage
GUI usage can be found [here](lightron-gui/README.md#usage).

**To start service of the web server in windows:**
1. Using GUI
* From Start Menu open Windows Services.
* Find the service named Lightron and right click on it and click the start button.<br>

2. Using Powershell
* Open Powershell with administrator privileges.
* Type the below command
    ```
    Start-Service Lightron-WebServer
    ```
    or
    ```
    net start Lightron-WebServer
    ```

**To start service of the web server in Linux:**
* Open Terminal.
* Type the below command
    ```
    sudo systemctl start lightrond
    ```
# Acknowledgements
* [@MoAlyousef](https://github.com/MoAlyousef)
* Amazing rust community at [here](https://discord.com/invite/yWGNDZ9F) and [here](https://discord.gg/rust-lang-community).
