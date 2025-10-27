package org.maplibre.test

/**
 * JNI wrapper for MapLibre Native core library.
 */
object MapLibreNative {
    init {
        System.loadLibrary("maplibre_android_test")
    }

    /**
     * Get the version of MapLibre Native.
     */
    external fun getVersion(): String

    /**
     * Test that the core library is accessible.
     * @return true if the core library is linked and working
     */
    external fun testCore(): Boolean
}
