package org.maplibre.test

import android.os.Bundle
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        val statusText = findViewById<TextView>(R.id.statusText)
        val versionText = findViewById<TextView>(R.id.versionText)

        try {
            // Test that the library loads and core is accessible
            val coreWorking = MapLibreNative.testCore()

            if (coreWorking) {
                // Get version information
                val version = MapLibreNative.getVersion()

                statusText.text = getString(R.string.success)
                versionText.text = version
            } else {
                statusText.text = "❌ Core library test failed"
                versionText.text = "N/A"
            }
        } catch (e: Exception) {
            statusText.text = "❌ Error loading library"
            versionText.text = e.message ?: "Unknown error"
        }
    }
}
