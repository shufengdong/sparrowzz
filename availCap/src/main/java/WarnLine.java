public class WarnLine {

    int loadState;
    String devName;
    String mRID;
    String substation;
    String switchName;
    String switchMRID;
    double maxI;
    double ratedCurrent;
    double loadRate;

    public int getLoadState() {
        return loadState;
    }

    public void setLoadState(int loadState) {
        this.loadState = loadState;
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

    public String getSwitchName() {
        return switchName;
    }

    public void setSwitchName(String switchName) {
        this.switchName = switchName;
    }

    public String getSwitchMRID() {
        return switchMRID;
    }

    public void setSwitchMRID(String switchMRID) {
        this.switchMRID = switchMRID;
    }

    public double getMaxI() {
        return maxI;
    }

    public void setMaxI(double maxI) {
        this.maxI = maxI;
    }

    public double getRatedCurrent() {
        return ratedCurrent;
    }

    public void setRatedCurrent(double ratedCurrent) {
        this.ratedCurrent = ratedCurrent;
    }

    public double getLoadRate() {
        return loadRate;
    }

    public void setLoadRate(double loadRate) {
        this.loadRate = loadRate;
    }
}
