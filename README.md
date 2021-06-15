# Lightron: Web Server
# Table Of Contents
- [About The Project](#about-the-project)
  * [Built With](#built-with)
- [Getting Started](#getting-started)
  * [Installation](#installation)
    + [Windows](#windows)
    + [Linux](#linux)
- [Usage](#usage)
  * [Hosting a website](#hosting-a-website)
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
1. Download lightron_setup.exe, double click it to launch the installer, and then follow the instructions.
2. You will be asked what directory to install .exe file in.
3. The installer will create an exe icon on the desktop and a menu-item under "All Programs"..<br>

### Linux
* Debian Based Distribution
    1. Download the .deb file by clicking here.
    2. Open Terminal and change the directory to where the debian package has been downloaded.
    3. Type the below command to install the package.
        ```
        sudo apt-get install ./lightron-0.1.0.deb
        ```
* Red Hat Based Distribution
    1. Download the .rpm file by clicking here.
    2. Open Terminal and change the directory to where the rpm package has been downloaded.
    3. Type the below command to install the package.
        ```
        sudo dnf install ./lightron-0.1.0-1.fc33.x86_64.rpm
        ```
* Arch Based Distribution
    1. Download the .pkg.tar.zst file by clicking here.
    2. Open Terminal and change the directory to where the arch package has been downloaded.
    3. Type the below command to install the package.
        ```
        sudo pacman -U ./lightron-0.1.0-1-x86_64.pkg.tar.zst
        ```
# Usage
GUI usage can be found [here](gui_readme.md#usage).

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
# Acknowledgement
* [@MoAlyousef](https://github.com/MoAlyousef)
* Amazing rust community at [here](https://discord.com/invite/yWGNDZ9F) and [here](https://discord.gg/rust-lang-community).
