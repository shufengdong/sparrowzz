package com.jinyun.cap;

public class LoadPosSw {
    String canIn;
    String swName;
    String swMRID;
    double[] newLoadI;   // 接入的负荷电流
    double[] swOrgLoad;   // 接入点开关原始负荷
    double[] swNewLoad; // 接入后开关电流
    double[] swRateI; // 接入点开关限额

    public String getCanIn() {
        return canIn;
    }

    public void setCanIn(String canIn) {
        this.canIn = canIn;
    }

    public String getSwName() {
        return swName;
    }

    public void setSwName(String swName) {
        this.swName = swName;
    }

    public String getSwMRID() {
        return swMRID;
    }

    public void setSwMRID(String swMRID) {
        this.swMRID = swMRID;
    }

    public double[] getNewLoadI() {
        return newLoadI;
    }

    public void setNewLoadI(double[] newLoadI) {
        this.newLoadI = newLoadI;
    }

    public double[] getSwOrgLoad() {
        return swOrgLoad;
    }

    public void setSwOrgLoad(double[] swOrgLoad) {
        this.swOrgLoad = swOrgLoad;
    }

    public double[] getSwNewLoad() {
        return swNewLoad;
    }

    public void setSwNewLoad(double[] swNewLoad) {
        this.swNewLoad = swNewLoad;
    }

    public double[] getSwRateI() {
        return swRateI;
    }

    public void setSwRateI(double[] swRateI) {
        this.swRateI = swRateI;
    }
}
