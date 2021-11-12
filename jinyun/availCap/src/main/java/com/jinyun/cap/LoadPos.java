package com.jinyun.cap;

import java.util.ArrayList;
import java.util.List;

public class LoadPos {
    double[] newLoad;   // 接入的负荷
    double[] newLoadI;  // 接入负荷的电流
    boolean[] canIn = new boolean[4];
    double[][] swOrgLoad = new double[4][];   // 接入点开关原始负荷
    double[][] swNewLoad = new double[4][96]; // 接入后开关电流
    double[] swRateI = new double[4]; // 接入点开关限额
    double[][] tfOrgLoad = new double[4][];   // 接入点公变原始负荷
    double[][] tfNewLoad = new double[4][96]; // 接入后公变负荷
    double[] tfRateCap = new double[4];   // 接入点公变额定容量
    List<Integer> seasons = new ArrayList<>(4); // 季节
    List<String> optEdges = new ArrayList<>(4); // 最优接入位置智能开关的mRID
    List<String> optEdgeNames = new ArrayList<>(4); // 最优接入位置智能开关的名称
    List<Double> seasonAvgCaps = new ArrayList<>(4);    // 负荷接入后，线路剩余可接入容量（电流）的日平均值
    List<Double> seasonMinCaps = new ArrayList<>(4);    // 负荷接入后，线路剩余可接入容量（电流）的日最小值
    List<Integer> seasons2 = new ArrayList<>(4);
    List<String> optEdges2 = new ArrayList<>(4);
    List<String> optEdgeNames2 = new ArrayList<>(4);
    List<Double> seasonAvgCaps2 = new ArrayList<>(4);
    List<Double> seasonMinCaps2 = new ArrayList<>(4);

    String[] optTfs = new String[4];   // 最优接入位置公变的mRID
    String[] optTfNames = new String[4];   // 最优接入位置公变的名称
    String[] optPhase = new String[4]; // 公变接入相别

    // season从0开始
    public void calSwNewLoad(int season) {
        for (int i = 0; i < 96; i++) {
            swNewLoad[season][i] = newLoadI[i] + swOrgLoad[season][i];
        }
    }

    public void calTfNewLoad(int season) {
        for (int i = 0; i < 96; i++) {
            tfNewLoad[season][i] = newLoad[i] + tfOrgLoad[season][i];
        }
    }

    public void setSwOrgLoad(int season, double[] swOrgSeasonLoad) {
        swOrgLoad[season] = swOrgSeasonLoad;
    }

    public void setSwNewLoad(int season, double[] swNewSeasonLoad) {
        swNewLoad[season] = swNewSeasonLoad;
    }

    public void setTfOrgLoad(int season, double[] tfOrgSeasonLoad) {
        tfOrgLoad[season] = tfOrgSeasonLoad;
    }

    public void setTfNewLoad(int season, double[] tfNewSeasonLoad) {
        tfNewLoad[season] = tfNewSeasonLoad;
    }

    public void setOptTf(int season, String optTf) {

    }

    public double[] getNewLoad() {
        return newLoad;
    }

    public void setNewLoad(double[] newLoad) {
        this.newLoad = newLoad;
    }

    public double[] getNewLoadI() {
        return newLoadI;
    }

    public void setNewLoadI(double[] newLoadI) {
        this.newLoadI = newLoadI;
    }

    public boolean[] getCanIn() {
        return canIn;
    }

    public void setCanIn(boolean[] canIn) {
        this.canIn = canIn;
    }

    public double[][] getSwOrgLoad() {
        return swOrgLoad;
    }

    public void setSwOrgLoad(double[][] swOrgLoad) {
        this.swOrgLoad = swOrgLoad;
    }

    public double[][] getSwNewLoad() {
        return swNewLoad;
    }

    public void setSwNewLoad(double[][] swNewLoad) {
        this.swNewLoad = swNewLoad;
    }

    public double[] getSwRateI() {
        return swRateI;
    }

    public void setSwRateI(double[] swRateI) {
        this.swRateI = swRateI;
    }

    public double[][] getTfOrgLoad() {
        return tfOrgLoad;
    }

    public void setTfOrgLoad(double[][] tfOrgLoad) {
        this.tfOrgLoad = tfOrgLoad;
    }

    public double[][] getTfNewLoad() {
        return tfNewLoad;
    }

    public void setTfNewLoad(double[][] tfNewLoad) {
        this.tfNewLoad = tfNewLoad;
    }

    public double[] getTfRateCap() {
        return tfRateCap;
    }

    public void setTfRateCap(double[] tfRateCap) {
        this.tfRateCap = tfRateCap;
    }

    public void addLoadPos(int season, String optEdge, String optEdgeName, double seasonAvgCap, double seasonMinCap) {
        this.seasons.add(season);
        this.optEdges.add(optEdge);
        this.optEdgeNames.add(optEdgeName);
        this.seasonAvgCaps.add(seasonAvgCap);
        this.seasonMinCaps.add(seasonMinCap);
    }

    public void addLoadPos2(int season, String optEdge, String optEdgeName, double seasonAvgCap, double seasonMinCap) {
        this.seasons2.add(season);
        this.optEdges2.add(optEdge);
        this.optEdgeNames2.add(optEdgeName);
        this.seasonAvgCaps2.add(seasonAvgCap);
        this.seasonMinCaps2.add(seasonMinCap);
    }

    public List<Integer> getSeasons() {
        return seasons;
    }

    public void setSeasons(List<Integer> seasons) {
        this.seasons = seasons;
    }

    public List<String> getOptEdges() {
        return optEdges;
    }

    public void setOptEdges(List<String> optEdges) {
        this.optEdges = optEdges;
    }

    public List<String> getOptEdgeNames() {
        return optEdgeNames;
    }

    public void setOptEdgeNames(List<String> optEdgeNames) {
        this.optEdgeNames = optEdgeNames;
    }

    public List<Double> getSeasonAvgCaps() {
        return seasonAvgCaps;
    }

    public void setSeasonAvgCaps(List<Double> seasonAvgCaps) {
        this.seasonAvgCaps = seasonAvgCaps;
    }

    public List<Double> getSeasonMinCaps() {
        return seasonMinCaps;
    }

    public void setSeasonMinCaps(List<Double> seasonMinCaps) {
        this.seasonMinCaps = seasonMinCaps;
    }

    public List<Integer> getSeasons2() {
        return seasons2;
    }

    public void setSeasons2(List<Integer> seasons2) {
        this.seasons2 = seasons2;
    }

    public List<String> getOptEdges2() {
        return optEdges2;
    }

    public void setOptEdges2(List<String> optEdges2) {
        this.optEdges2 = optEdges2;
    }

    public List<String> getOptEdgeNames2() {
        return optEdgeNames2;
    }

    public void setOptEdgeNames2(List<String> optEdgeNames2) {
        this.optEdgeNames2 = optEdgeNames2;
    }

    public List<Double> getSeasonAvgCaps2() {
        return seasonAvgCaps2;
    }

    public void setSeasonAvgCaps2(List<Double> seasonAvgCaps2) {
        this.seasonAvgCaps2 = seasonAvgCaps2;
    }

    public List<Double> getSeasonMinCaps2() {
        return seasonMinCaps2;
    }

    public void setSeasonMinCaps2(List<Double> seasonMinCaps2) {
        this.seasonMinCaps2 = seasonMinCaps2;
    }

    public String[] getOptTfs() {
        return optTfs;
    }

    public void setOptTfs(String[] optTfs) {
        this.optTfs = optTfs;
    }

    public String[] getOptTfNames() {
        return optTfNames;
    }

    public void setOptTfNames(String[] optTfNames) {
        this.optTfNames = optTfNames;
    }

    public String[] getOptPhase() {
        return optPhase;
    }

    public void setOptPhase(String[] optPhase) {
        this.optPhase = optPhase;
    }
}
