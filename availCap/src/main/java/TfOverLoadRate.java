public class TfOverLoadRate {

    String feederName;
    String devName;
    String mRID;
    String lineName;
    String lineMRID;
    String substation;
    double[] overLoadRate;

    public String getFeederName() {
        return feederName;
    }

    public void setFeederName(String feederName) {
        this.feederName = feederName;
    }

    public String getDevName() {
        return devName;
    }

    public void setDevName(String devName) {
        this.devName = devName;
    }

    public String getmRID() {
        return mRID;
    }

    public void setmRID(String mRID) {
        this.mRID = mRID;
    }

    public String getSubstation() {
        return substation;
    }

    public void setSubstation(String substation) {
        this.substation = substation;
    }

    public String getLineName() {
        return lineName;
    }

    public void setLineName(String lineName) {
        this.lineName = lineName;
    }

    public String getLineMRID() {
        return lineMRID;
    }

    public void setLineMRID(String lineMRID) {
        this.lineMRID = lineMRID;
    }

    public double[] getOverLoadRate() {
        return overLoadRate;
    }

    public void setOverLoadRate(double[] overLoadRate) {
        this.overLoadRate = overLoadRate;
    }
}
