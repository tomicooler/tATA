**NOTE**
  
I built tATA a long time ago (2014.07.13 - 2018.11.06), the code was never pushed to a public git repository before.
The content of this directory represents the latest state of the Watcher SMS application. Just the build scripts were
updated and the handling of secrets were changed, so I could publish the source code here.


Downloads: [Original Release](https://github.com/tomicooler/tATAPowerDetector)


**Build instructions**

Create a Firebase project add an android application with the proper package names (probably it should be changed).
Then download the **google-services.json** to WatcherSMS/google-services.json from your Firebase console, and set
your Google Maps API key in the WatcherSMS/src/main/res/values/api\_keys.xml file.

Build the project with Android Studio.

