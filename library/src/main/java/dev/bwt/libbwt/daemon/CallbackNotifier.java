package dev.bwt.libbwt.daemon;

public interface CallbackNotifier {
    void onBooting(long shutdownPtr);
    void onSyncProgress(float progress, int tip);
    void onScanProgress(float progress, int eta);
    void onElectrumReady(String addr);
    void onHttpReady(String addr);
    void onReady();
}
