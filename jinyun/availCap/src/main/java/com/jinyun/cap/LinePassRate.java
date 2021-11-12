package com.jinyun.cap;

public class LinePassRate {

    String feederName;
    String substation;
    double[] passRate;

    public String getFeederName() {
        return feederName;
    }

    public void setFeederName(String feederName) {
        this.feederName = feederName;
    }

    public String getSubstation() {
        return substation;
    }

    public void setSubstation(String substation) {
        this.substation = substation;
    }

    public double[] getPassRate() {
        return passRate;
    }

    public void setPassRate(double[] passRate) {
        this.passRate = passRate;
    }
}
