package com.jinyun.cap;

public class LoadPosSeason {

    int loadId;
    String substation;
    String feederName;
    String canIn;
    String tfName;   // 最优接入位置公变的mRID
    String tfMRID;   // 最优接入位置公变的名称
    String phase; // 公变接入相别
    String time;

    public int getLoadId() {
        return loadId;
    }

    public void setLoadId(int loadId) {
        this.loadId = loadId;
    }

    public String getSubstation() {
        return substation;
    }

    public void setSubstation(String substation) {
        this.substation = substation;
    }

    public String getFeederName() {
        return feederName;
    }

    public void setFeederName(String feederName) {
        this.feederName = feederName;
    }

    public String getCanIn() {
        return canIn;
    }

    public void setCanIn(String canIn) {
        this.canIn = canIn;
    }

    public String getTfName() {
        return tfName;
    }

    public void setTfName(String tfName) {
        this.tfName = tfName;
    }

    public String getTfMRID() {
        return tfMRID;
    }

    public void setTfMRID(String tfMRID) {
        this.tfMRID = tfMRID;
    }

    public String getPhase() {
        return phase;
    }

    public void setPhase(String phase) {
        this.phase = phase;
    }

    public String getTime() {
        return time;
    }

    public void setTime(String time) {
        this.time = time;
    }
}
