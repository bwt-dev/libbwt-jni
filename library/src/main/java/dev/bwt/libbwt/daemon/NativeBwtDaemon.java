package dev.bwt.libbwt.daemon;

public class NativeBwtDaemon {
    static {
        System.loadLibrary("bwt_jni");
    }

    // Start the bwt daemon with the configured server(s)
    // Blocks the current thread until the daemon is stopped.
    public static native long start(String jsonConfig, CallbackNotifier callback);

    // Shutdown thw bwt daemon
    public static native void shutdown(long shutdownPtr);
}