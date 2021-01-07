package dev.bwt.libbwt.daemon

import android.util.Log
import com.google.gson.Gson
import com.google.gson.annotations.SerializedName
import java.util.*

class BwtDaemon(
    var config: BwtConfig,
) {
    var started: Boolean = false
    var terminate: Boolean = false
    var shutdownPtr: Long? = null
    var electrumAddr: String? = null
    var httpAddr: String? = null

    fun start(callback: ProgressNotifier) {
        Log.v("bwt-daemon","starting")
        val jsonConfig = Gson().toJson(config)
        NativeBwtDaemon.start(jsonConfig, object : CallbackNotifier {
            override fun onBooting() {
                Log.v("bwt-daemon", "booting")
                if (!terminate) callback.onBooting()
            }

            override fun onSyncProgress(progress: Float, tipUnix: Int) {
                val tipDate = Date(tipUnix.toLong() * 1000)
                Log.v("bwt-daemon", "sync progress ${progress * 100}%")
                if (!started && !terminate) callback.onSyncProgress(progress, tipDate)
            }

            override fun onScanProgress(progress: Float, eta: Int) {
                Log.v("bwt-daemon", "scan progress ${progress * 100}%")
                if (!started && !terminate) callback.onScanProgress(progress, eta)
            }

            override fun onElectrumReady(addr: String) {
                Log.v("bwt-daemon", "electrum ready on $addr")
                electrumAddr = addr
            }

            override fun onHttpReady(addr: String) {
                Log.v("bwt-daemon", "http ready on $addr")
                httpAddr = addr
            }

            override fun onReady(shutdownPtr_: Long) {
                Log.v("bwt-daemon", "bwt is ready")
                started = true
                shutdownPtr = shutdownPtr_
                if (!terminate) callback.onReady(this@BwtDaemon)
                else shutdown()
            }
        })
    }

    fun shutdown() {
        Log.v("bwt-daemon","shutdown $shutdownPtr")
        if (shutdownPtr != null) {
            NativeBwtDaemon.shutdown(shutdownPtr!!)
            shutdownPtr = null
        } else {
            // We cannot shutdown yet, mark the daemon for termination when it becomes possible
            terminate = true
        }
    }
}

interface ProgressNotifier {
    fun onBooting() {};
    fun onSyncProgress(progress: Float, tip: Date) {};
    fun onScanProgress(progress: Float, eta: Int) {};
    fun onReady(bwt: BwtDaemon) {};
}

data class BwtConfig(
    @SerializedName("network") var network: String? = null,
    @SerializedName("bitcoind_url") var bitcoindUrl: String? = null,
    @SerializedName("bitcoind_auth") var bitcoindAuth: String? = null,
    @SerializedName("bitcoind_dir") var bitcoindDir: String? = null,
    @SerializedName("bitcoind_cookie") var bitcoindCookie: String? = null,
    @SerializedName("bitcoind_wallet") var bitcoindWallet: String? = null,
    @SerializedName("descriptors") var descriptors: Array<String>? = null,
    @SerializedName("xpubs") var xpubs: Array<String>? = null,
    @SerializedName("rescan_since") var rescanSince: Int? = null,
    @SerializedName("gap_limit") var gapLimit: Int? = null,
    @SerializedName("initial_import_size") var initialImportSize: Int? = null,
    @SerializedName("poll_interval") var pollInterval: Array<Int>? = null,
    @SerializedName("verbose") var verbose: Int? = null,
    @SerializedName("tx_broadcast_cmt") var txBroadcastCmd: String? = null,
    @SerializedName("electrum_addr") var electrumAddr: String? = null,
    @SerializedName("electrum_skip_merkle") var electrumSkipMerkle: Boolean? = null,
    @SerializedName("http_addr") var httpAddr: String? = null,
    @SerializedName("http_cors") var httpCors: Boolean? = null,
    @SerializedName("webhooks_urls") var webhookUrls: Array<String>? = null,
    @SerializedName("unix_listener_path") var unixListenerPath: String? = null,
    @SerializedName("require_addresses") var requireAddresses: Boolean? = null,
    @SerializedName("setup_logger") var setupLogger: Boolean? = null,
    @SerializedName("force_rescan") var forceRescan: Boolean? = null,
) {}