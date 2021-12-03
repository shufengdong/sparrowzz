package com.jinyun.web.service;

import com.jinyun.cap.*;
import com.jinyun.web.entity.ImportData;
import com.jinyun.web.entity.LinePassRateWeb;
import com.jinyun.web.entity.TfOverLoadRateWeb;
import org.springframework.stereotype.Service;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileNotFoundException;
import java.util.*;

@Service("CapService")
public class CapService {
    static String allFeederNameTable = "所有馈线名称";
    static String allPsWarnTable = "所有馈线预警设备";
    static String allPsLineWarnTable = "所有馈线线路预警设备";
    static String allPsTfWarnTable = "所有馈线公变预警设备";
    static String maxMinAvailCapTable = "最大最小可开放容量线路";
    static String lineITableName = "电流";
    static String switchTableName = "开关";
    static String lineParamTableName = "参数";
    static String oneLineParamTableName = "单线参数";
    static String transformerTableName = "公变";
    static String tfParamTableName = "公变参数";
    static String availCapTableName = "可接入容量";
    static String swToLineTableName = "开关线路对应";
    static String swToTfTableName = "开关公变对应";
    static String tfToLineTableName = "公变线路对应";
    static String lineWarnTableName = "线路预警";
    static String tfWarnTableName = "公变预警";
    static String substationTableName = "变电站";
    static String tfMonthUbTableName = "公变月不平衡度";
    static String linePassRateTableName = "线路合格率";
    static String tfOverLoadRateTableName = "公变越限率";
    static String feederTableName = "馈线名称";
    static String loadPosTable = "负荷接入位置";
    static String allPsDbFile = "D:\\others\\yunqi\\项目\\缙云项目\\test\\allPsDb\\allPs.db";
    static String feederDbFile = "D:\\others\\yunqi\\项目\\缙云项目\\test\\feederDb";
    static String feederDbName = "溪南G134线";
    static String cimFile = "D:\\others\\yunqi\\项目\\缙云项目\\test\\CIM";

    public Map<String, Object> gridStatistics() {
        Map<String,Object> result = new HashMap<>();
        result.put("110StationCount",25);
        result.put("35StationCount",36);
        result.put("10kVBusCount",67);
        result.put("busCount",156);
        result.put("transformerCount",300);
        return result;
    }

    public Map<String, Object> lowMarginWarnStatistics() {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        int hardLineNum = sqliteDb.queryWarnDevNum(allPsWarnTable, 1, 1);
        int overLineNum = sqliteDb.queryWarnDevNum(allPsWarnTable, 2, 1);
        int hardTfNum = sqliteDb.queryWarnDevNum(allPsWarnTable, 1, 2);
        int overTfNum = sqliteDb.queryWarnDevNum(allPsWarnTable, 2, 2);
        Map<String,Object> result = new HashMap<>();
        result.put("hardLineNum",hardLineNum);
        result.put("overLineNum",overLineNum);
        result.put("hardTfNum",hardTfNum);
        result.put("overTfNum",overTfNum);
        return result;
    }

    public Map<String, Object> transformerUnbalance() {
        SqliteDb allPsDb = new SqliteDb(allPsDbFile);
        List<String> feeders = allPsDb.queryAllFeederName(allFeederNameTable);  // 查询馈线名称
        List<TfUb> tfUbs = new LinkedList<>();
        for (String feeder : feeders) {
            SqliteDb sqliteDb = new SqliteDb(feederDbFile + "\\" + feeder + ".db");
            tfUbs.addAll(sqliteDb.queryTfMonthUb(feeder + tfMonthUbTableName));
        }
        Map<String,Object> result = new HashMap<>();
        int count = 0;
        for(TfUb tfUb:tfUbs){
            if(count == 5) break;
            if(result.containsKey(tfUb.getDevName()))
                continue;
            result.put(tfUb.getDevName(),tfUb.getUb());
            count ++;
        }
        return result;
    }

    public Map<String,Object> transformerUnbalanceList(int page, int rows) {
        SqliteDb allPsDb = new SqliteDb(allPsDbFile);
        List<String> feeders = allPsDb.queryAllFeederName(allFeederNameTable);  // 查询馈线名称
        List<TfUb> tfUbs = new LinkedList<>();
        int startRow = (page-1)*rows;
        int endRow = startRow+rows;
        for (String feeder : feeders) {
            SqliteDb sqliteDb = new SqliteDb(feederDbFile + "\\" + feeder + ".db");
            List<TfUb> c = sqliteDb.queryTfMonthUb(feeder + tfMonthUbTableName);
            tfUbs.addAll(c);;
        }
        List<TfUb> result = new LinkedList<>();
        for(int i=startRow;i<endRow;i++){
            result.add(tfUbs.get(i));
        }
        Map<String,Object> result1 = new HashMap<>();
        result1.put("total",tfUbs.size());
        result1.put("rows",result);
        return result1;
    }

    public int hardLineListCount() {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        int total = sqliteDb.queryWarnLineCount(allPsLineWarnTable, 1);
        return total;
    }

    public List hardLineList(int page, int rows) {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        List<WarnLine> hardLine = sqliteDb.queryWarnLine(allPsLineWarnTable, 1,page,rows);
        return hardLine;
    }


    public int hardTransformerListCount() {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        int total = sqliteDb.queryWarnTfCount(allPsTfWarnTable, 1);
        return total;
    }

    public List hardTransformerList(int page, int rows) {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        List<WarnTf> hardTf = sqliteDb.queryWarnTf(allPsTfWarnTable, 1,page,rows);
        return hardTf;
    }

    public int overLineListCount() {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        int count = sqliteDb.queryWarnLineCount(allPsLineWarnTable, 2);
        return count;
    }

    public List overLineList(int page, int rows) {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        List<WarnLine> overLine = sqliteDb.queryWarnLine(allPsLineWarnTable, 2,page,rows);
        return overLine;
    }

    public int overTransformerListCount() {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        int count = sqliteDb.queryWarnTfCount(allPsTfWarnTable, 2);
        return count;
    }

    public List overTransformerList(int page, int rows) {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        List<WarnTf> overTf = sqliteDb.queryWarnTf(allPsTfWarnTable, 2,page,rows);
        return overTf;
    }

    public List linePassRate() {
        SqliteDb sqliteDb = new SqliteDb(feederDbFile + "\\" + feederDbName + ".db");
        List<LinePassRate> linePassRates = sqliteDb.queryLinePassRate(feederDbName + linePassRateTableName);
        List<LinePassRateWeb> linePassRateWebs = new ArrayList<>();
        for(LinePassRate it:linePassRates){
            LinePassRateWeb linePassRateWeb = new LinePassRateWeb(it.getFeederName(),
                    it.getSubstation(),it.getPassRate()[0],it.getPassRate()[1],it.getPassRate()[2],it.getPassRate()[3]);
            linePassRateWebs.add(linePassRateWeb);
        }
        return linePassRateWebs;
    }

    public Object transformerOverLoadRate() {
        SqliteDb allPsDb = new SqliteDb(allPsDbFile);
        List<String> feeders = allPsDb.queryAllFeederName(allFeederNameTable);  // 查询馈线名称
        List<TfOverLoadRate> tfOverLoadRates = new LinkedList<>();
        for (String feeder : feeders) {
            SqliteDb sqliteDb = new SqliteDb(feederDbFile + "\\" + feeder + ".db");
            tfOverLoadRates.addAll(sqliteDb.queryTfOverLoadRate(feeder + tfOverLoadRateTableName));
        }
        TfOverLoadRate it = tfOverLoadRates.get(0); //todo 先取第一个
        TfOverLoadRateWeb tfOverLoadRateWeb = new TfOverLoadRateWeb(it.getFeederName(),it.getDevName(),it.getmRID(),it.getLineName(),it.getLineMRID(),
                it.getSubstation(),it.getOverLoadRate()[0],it.getOverLoadRate()[1],it.getOverLoadRate()[2],it.getOverLoadRate()[3]);
        return tfOverLoadRateWeb;
    }

    public Map<String, Object> maxMinAvailCap() {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        MaxMinAvailCap maxFeeder = sqliteDb.queryMaxMinAvailCap(maxMinAvailCapTable, 1);
        MaxMinAvailCap minFeeder = sqliteDb.queryMaxMinAvailCap(maxMinAvailCapTable, 2);
        Map<String,Object> result = new HashMap<>();
        result.put("max",maxFeeder);
        result.put("min",minFeeder);
        return result;
    }

    public Map<String, Object> baseInfo(String mRID) {
        SqliteDb sqliteDb = new SqliteDb(feederDbFile + "\\" + feederDbName + ".db");
        double lineRatedI = sqliteDb.queryOneLineParam(feederDbName + oneLineParamTableName, mRID);
        double lineMaxI = sqliteDb.queryMaxSwitchI(feederDbName + switchTableName + HistoryData.seasonTable, mRID, -1);
        double avgI = sqliteDb.queryMaxSwitchI(feederDbName + switchTableName + HistoryData.seasonTable, mRID, -2);
        Map<String,Object> result = new HashMap<>();
        result.put("lineRatedI",lineRatedI);
        result.put("lineMaxI",lineMaxI);
        result.put("avgI",avgI);
        return result;
    }

    public Map<String, Object> lineSummer(String mRID) {
        SqliteDb sqliteDb = new SqliteDb(feederDbFile + "\\" + feederDbName + ".db");
        // lineRatedICv为线路限额，seasonCluster2为负荷聚类曲线，seasonMax2为最大负荷曲线，cap2为可开放容量
        double lineRatedI = sqliteDb.queryOneLineParam(feederDbName + oneLineParamTableName, mRID);
        double[] lineRatedICv = new double[96];
        for (int i = 0; i < 96; i++) {
            lineRatedICv[i] = lineRatedI;
        }
        double[] seasonCluster2 = sqliteDb.querySeasonSwitchI(feederDbName + switchTableName + HistoryData.seasonClusterTable, mRID, 2, 96);
        double[] seasonMax2 = sqliteDb.querySeasonSwitchI(feederDbName + switchTableName + HistoryData.seasonTable, mRID, 2, 96);
        double[] cap2 = sqliteDb.queryAvailCap(feederDbName + availCapTableName, mRID, 2, 96);

        Map<String,Object> result = new HashMap<>();
        result.put("lineRatedICv",lineRatedICv);
        result.put("seasonCluster2",seasonCluster2);
        result.put("seasonMax2",seasonMax2);
        result.put("cap2",cap2);
        return result;
    }

    public Map<String, Object> lineWinter(String mRID) {
        SqliteDb sqliteDb = new SqliteDb(feederDbFile + "\\" + feederDbName + ".db");
        double lineRatedI = sqliteDb.queryOneLineParam(feederDbName + oneLineParamTableName, mRID);
        double[] lineRatedICv = new double[96];
        for (int i = 0; i < 96; i++) {
            lineRatedICv[i] = lineRatedI;
        }
        double[] seasonCluster4 = sqliteDb.querySeasonSwitchI(feederDbName + switchTableName + HistoryData.seasonClusterTable, mRID, 4, 96);
        double[] seasonMax4 = sqliteDb.querySeasonSwitchI(feederDbName + switchTableName + HistoryData.seasonTable, mRID, 4, 96);
        double[] cap4 = sqliteDb.queryAvailCap(feederDbName + availCapTableName, mRID, 4, 96);
        Map<String,Object> result = new HashMap<>();
        result.put("lineRatedICv",lineRatedICv);
        result.put("seasonCluster4",seasonCluster4);
        result.put("seasonMax4",seasonMax4);
        result.put("cap4",cap4);
        return result;
    }

    public Map<String, Object> linePs(String mRID) {
        SqliteDb sqliteDb = new SqliteDb(feederDbFile + "\\" + feederDbName + ".db");
        double lineRatedI = sqliteDb.queryOneLineParam(feederDbName + oneLineParamTableName, mRID);
        double[] lineRatedICv = new double[96];
        for (int i = 0; i < 96; i++) {
            lineRatedICv[i] = lineRatedI;
        }
        double[] psCluster2 = sqliteDb.querySeasonSwitchI(feederDbName + switchTableName + HistoryData.psClusterTable, mRID, 2, 96);
        double[] seasonMax2 = sqliteDb.querySeasonSwitchI(feederDbName + switchTableName + HistoryData.seasonTable, mRID, 2, 96);
        double[] cap2 = sqliteDb.queryAvailCap(feederDbName + availCapTableName, mRID, 2, 96);
        Map<String,Object> result = new HashMap<>();
        result.put("lineRatedICv",lineRatedICv);
        result.put("psCluster2",psCluster2);
        result.put("seasonMax2",seasonMax2);
        result.put("cap2",cap2);
        return result;
    }

    public int loadPosListCout() {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        int total = sqliteDb.queryLoadPosSeasonCount(loadPosTable, 1);
        return total;
    }

    public List loadPosList(int page, int rows) {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        List<LoadPosSeason> loadPosSeason = sqliteDb.queryLoadPosSeason(loadPosTable, 1,page,rows);
        return loadPosSeason;
    }

    public Object loadPosSw(String mRID) {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        LoadPosSw loadPosSw = sqliteDb.queryLoadPosSw(loadPosTable, Integer.parseInt(mRID), 1);
        return loadPosSw;
    }

    public Object loadPosTf(String mRID) {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        LoadPosTf loadPosTf = sqliteDb.queryLoadPosTf(loadPosTable, Integer.parseInt(mRID), 1);
        return loadPosTf;
    }

    public Map<String, Object> lineInfo(String mRID) {
        // mouseOverSw为鼠标放置在开关上，查询线路限额，最大电流，可开放容量。args[1]为馈线数据库文件夹的路径，feederDbName为馈线名称，mRID为线路mRID
        SqliteDb sqliteDb = new SqliteDb(feederDbFile + "\\" + feederDbName + ".db");
        double lineRatedI = sqliteDb.queryOneLineParam(feederDbName + oneLineParamTableName, mRID);
        double lineMaxI = sqliteDb.queryMaxSwitchI(feederDbName + switchTableName + HistoryData.seasonTable, mRID, -1);
        double[] cap1 = sqliteDb.queryAvailCap(feederDbName + availCapTableName, mRID, 1, 96);
        double[] cap2 = sqliteDb.queryAvailCap(feederDbName + availCapTableName, mRID, 2, 96);
        double[] cap3 = sqliteDb.queryAvailCap(feederDbName + availCapTableName, mRID, 3, 96);
        double[] cap4 = sqliteDb.queryAvailCap(feederDbName + availCapTableName, mRID, 4, 96);
        Map<String,Object> result = new HashMap<>();
        result.put("lineRatedI",lineRatedI);
        result.put("lineMaxI",lineMaxI);
        result.put("cap1",cap1);
        result.put("cap2",cap2);
        result.put("cap3",cap3);
        result.put("cap4",cap4);
        return result;
    }

    public Map<String, Object> transformerInfo(String mRID) {
        // mouseOverTF为鼠标放置在公变上，查询公变容量，最大负荷，三相不平衡度。args[1]为馈线数据库文件夹的路径，feederDbName为馈线名称，cimFile为公变mRID
        SqliteDb sqliteDb = new SqliteDb(feederDbFile + "\\" + feederDbName + ".db");
        double tFRatedCap = sqliteDb.queryTFCap(feederDbName + tfParamTableName, mRID);
        double tFMaxP = sqliteDb.queryMaxTFP(feederDbName + transformerTableName + HistoryData.seasonTable, mRID, -1);
        double[] ub = sqliteDb.queryTFUb(feederDbName + transformerTableName + HistoryData.unbalanceTable, mRID, -1);
        // 低压负荷接入相别
        String phase = sqliteDb.queryMinIPhase(feederDbName + transformerTableName + HistoryData.minITable, mRID, -1);
        Map<String,Object> result = new HashMap<>();
        result.put("tFRatedCap",tFRatedCap);
        result.put("tFMaxP",tFMaxP);
        result.put("ub",ub);
        result.put("phase",phase);
        return result;
    }

    public Map<String, Object> lineInfoDetail(String mRID) {
        // mouseClickSw为鼠标点击开关上，查询线路限额，最大电流，可开放容量。args[1]为馈线数据库文件夹的路径，feederDbName为馈线名称，mRID为开关mRID
        SqliteDb sqliteDb = new SqliteDb(feederDbFile + "\\" + feederDbName + ".db");
        double lineRatedI = sqliteDb.queryOneLineParam(feederDbName + oneLineParamTableName, mRID);
        double lineMaxI = sqliteDb.queryMaxSwitchI(feederDbName + switchTableName + HistoryData.seasonTable, mRID, -1);
        double lineAvgI = sqliteDb.queryMaxSwitchI(feederDbName + switchTableName + HistoryData.seasonTable, mRID, -2);
        double[] cap1 = sqliteDb.queryAvailCap(feederDbName + availCapTableName, mRID, 1, 96);
        double[] cap2 = sqliteDb.queryAvailCap(feederDbName + availCapTableName, mRID, 2, 96);
        double[] cap3 = sqliteDb.queryAvailCap(feederDbName + availCapTableName, mRID, 3, 96);
        double[] cap4 = sqliteDb.queryAvailCap(feederDbName + availCapTableName, mRID, 4, 96);
        double[] seasonCluster1 = sqliteDb.querySeasonSwitchI(feederDbName + switchTableName + HistoryData.seasonClusterTable, mRID, 1, 96);
        double[] seasonCluster2 = sqliteDb.querySeasonSwitchI(feederDbName + switchTableName + HistoryData.seasonClusterTable, mRID, 2, 96);
        double[] seasonCluster3 = sqliteDb.querySeasonSwitchI(feederDbName + switchTableName + HistoryData.seasonClusterTable, mRID, 3, 96);
        double[] seasonCluster4 = sqliteDb.querySeasonSwitchI(feederDbName + switchTableName + HistoryData.seasonClusterTable, mRID, 4, 96);
        double[] psCluster1 = sqliteDb.querySeasonSwitchI(feederDbName + switchTableName + HistoryData.psClusterTable, mRID, 1, 96);
        double[] psCluster2 = sqliteDb.querySeasonSwitchI(feederDbName + switchTableName + HistoryData.psClusterTable, mRID, 2, 96);
        double[] psCluster3 = sqliteDb.querySeasonSwitchI(feederDbName + switchTableName + HistoryData.psClusterTable, mRID, 3, 96);
        double[] psCluster4 = sqliteDb.querySeasonSwitchI(feederDbName + switchTableName + HistoryData.psClusterTable, mRID, 4, 96);
        double[] seasonMax1 = sqliteDb.querySeasonSwitchI(feederDbName + switchTableName + HistoryData.seasonTable, mRID, 1, 96);
        double[] seasonMax2 = sqliteDb.querySeasonSwitchI(feederDbName + switchTableName + HistoryData.seasonTable, mRID, 2, 96);
        double[] seasonMax3 = sqliteDb.querySeasonSwitchI(feederDbName + switchTableName + HistoryData.seasonTable, mRID, 3, 96);
        double[] seasonMax4 = sqliteDb.querySeasonSwitchI(feederDbName + switchTableName + HistoryData.seasonTable, mRID, 4, 96);
        for (int i = 0; i < 96; i++) {
            System.out.println(lineRatedI + "," + seasonCluster1[i] + "," + seasonMax1[i] + "," + seasonCluster2[i] + "," + seasonMax2[i] + "," +
                    seasonCluster3[i] + "," + seasonMax3[i] + "," + seasonCluster4[i] + "," + seasonMax4[i]);
        }
        System.out.println();
        for (int i = 0; i < 96; i++) {
            System.out.println(lineRatedI + "," + psCluster1[i] + "," + seasonMax1[i] + "," + psCluster2[i] + "," + seasonMax2[i] + "," +
                    psCluster3[i] + "," + seasonMax3[i] + "," + psCluster4[i] + "," + seasonMax4[i]);
        }
        Map<String,Object> result = new HashMap<>();
        result.put("lineRatedI",lineRatedI);
        result.put("lineMaxI",lineMaxI);
        result.put("lineAvgI",lineAvgI);
        result.put("cap1",cap1);
        result.put("cap2",cap2);
        result.put("cap3",cap3);
        result.put("cap4",cap4);
        result.put("seasonCluster1",seasonCluster1);
        result.put("seasonCluster2",seasonCluster2);
        result.put("seasonCluster3",seasonCluster3);
        result.put("seasonCluster4",seasonCluster4);
        result.put("psCluster1",psCluster1);
        result.put("psCluster2",psCluster2);
        result.put("psCluster3",psCluster3);
        result.put("psCluster4",psCluster4);
        result.put("seasonMax1",seasonMax1);
        result.put("seasonMax2",seasonMax2);
        result.put("seasonMax3",seasonMax3);
        result.put("seasonMax4",seasonMax4);
        return result;
    }

    public Map<String, Object> transformerInfoDetail(String mRID) {
        // mouseClickTF为鼠标点击公变上，查询公变容量，最大负荷，三相不平衡度。args[1]为馈线数据库文件夹的路径，feederDbName为馈线名称，mRID为公变mRID
        SqliteDb sqliteDb = new SqliteDb(feederDbFile + "\\" + feederDbName + ".db");
        double tFRatedCap = sqliteDb.queryTFCap(feederDbName + tfParamTableName, mRID);
        double tFMaxI = sqliteDb.queryMaxTFP(feederDbName + transformerTableName + HistoryData.seasonTable, mRID, -2);
        double[] ub = sqliteDb.queryTFUb(feederDbName + transformerTableName + HistoryData.unbalanceTable, mRID, -1);
        // 负荷
        double[] seasonMax1 = sqliteDb.querySeasonTFP(feederDbName + transformerTableName + HistoryData.seasonTable, mRID, 1, 96);
        double[] seasonMax2 = sqliteDb.querySeasonTFP(feederDbName + transformerTableName + HistoryData.seasonTable, mRID, 2, 96);
        double[] seasonMax3 = sqliteDb.querySeasonTFP(feederDbName + transformerTableName + HistoryData.seasonTable, mRID, 3, 96);
        double[] seasonMax4 = sqliteDb.querySeasonTFP(feederDbName + transformerTableName + HistoryData.seasonTable, mRID, 4, 96);
        double[] seasonCluster1 = sqliteDb.querySeasonTFP(feederDbName + transformerTableName + HistoryData.seasonClusterTable, mRID, 1, 96);
        double[] seasonCluster2 = sqliteDb.querySeasonTFP(feederDbName + transformerTableName + HistoryData.seasonClusterTable, mRID, 2, 96);
        double[] seasonCluster3 = sqliteDb.querySeasonTFP(feederDbName + transformerTableName + HistoryData.seasonClusterTable, mRID, 3, 96);
        double[] seasonCluster4 = sqliteDb.querySeasonTFP(feederDbName + transformerTableName + HistoryData.seasonClusterTable, mRID, 4, 96);
        double[] psCluster1 = sqliteDb.querySeasonTFP(feederDbName + transformerTableName + HistoryData.psClusterTable, mRID, 1, 96);
        double[] psCluster2 = sqliteDb.querySeasonTFP(feederDbName + transformerTableName + HistoryData.psClusterTable, mRID, 2, 96);
        double[] psCluster3 = sqliteDb.querySeasonTFP(feederDbName + transformerTableName + HistoryData.psClusterTable, mRID, 3, 96);
        double[] psCluster4 = sqliteDb.querySeasonTFP(feederDbName + transformerTableName + HistoryData.psClusterTable, mRID, 4, 96);
        // 三相不平衡度
        double[] seasonMaxUb1 = sqliteDb.querySeasonTFUb(feederDbName + transformerTableName + HistoryData.ubMaxTable, mRID, 1, 96);
        double[] seasonMaxUb2 = sqliteDb.querySeasonTFUb(feederDbName + transformerTableName + HistoryData.ubMaxTable, mRID, 2, 96);
        double[] seasonMaxUb3 = sqliteDb.querySeasonTFUb(feederDbName + transformerTableName + HistoryData.ubMaxTable, mRID, 3, 96);
        double[] seasonMaxUb4 = sqliteDb.querySeasonTFUb(feederDbName + transformerTableName + HistoryData.ubMaxTable, mRID, 4, 96);
        double[] seasonClusterUb1 = sqliteDb.querySeasonTFUb(feederDbName + transformerTableName + HistoryData.seasonClusterTable, mRID, 1, 96);
        double[] seasonClusterUb2 = sqliteDb.querySeasonTFUb(feederDbName + transformerTableName + HistoryData.seasonClusterTable, mRID, 2, 96);
        double[] seasonClusterUb3 = sqliteDb.querySeasonTFUb(feederDbName + transformerTableName + HistoryData.seasonClusterTable, mRID, 3, 96);
        double[] seasonClusterUb4 = sqliteDb.querySeasonTFUb(feederDbName + transformerTableName + HistoryData.seasonClusterTable, mRID, 4, 96);
        double[] psClusterUb1 = sqliteDb.querySeasonTFUb(feederDbName + transformerTableName + HistoryData.psClusterTable, mRID, 1, 96);
        double[] psClusterUb2 = sqliteDb.querySeasonTFUb(feederDbName + transformerTableName + HistoryData.psClusterTable, mRID, 2, 96);
        double[] psClusterUb3 = sqliteDb.querySeasonTFUb(feederDbName + transformerTableName + HistoryData.psClusterTable, mRID, 3, 96);
        double[] psClusterUb4 = sqliteDb.querySeasonTFUb(feederDbName + transformerTableName + HistoryData.psClusterTable, mRID, 4, 96);
        // 公变可开放容量
        double[] cap1 = sqliteDb.queryAvailCap(feederDbName + transformerTableName + HistoryData.tfAvailCapTable, mRID, 1, 96);
        double[] cap2 = sqliteDb.queryAvailCap(feederDbName + transformerTableName + HistoryData.tfAvailCapTable, mRID, 2, 96);
        double[] cap3 = sqliteDb.queryAvailCap(feederDbName + transformerTableName + HistoryData.tfAvailCapTable, mRID, 3, 96);
        double[] cap4 = sqliteDb.queryAvailCap(feederDbName + transformerTableName + HistoryData.tfAvailCapTable, mRID, 4, 96);
        Map<String,Object> result = new HashMap<>();
        result.put("tFRatedCap",tFRatedCap);
        result.put("tFMaxI",tFMaxI);
        result.put("ub",ub);
        result.put("seasonMax1",seasonMax1);
        result.put("seasonMax2",seasonMax2);
        result.put("seasonMax3",seasonMax3);
        result.put("seasonMax4",seasonMax4);
        result.put("seasonCluster1",seasonCluster1);
        result.put("seasonCluster2",seasonCluster2);
        result.put("seasonCluster3",seasonCluster3);
        result.put("seasonCluster4",seasonCluster4);
        result.put("psCluster1",psCluster1);
        result.put("psCluster2",psCluster2);
        result.put("psCluster3",psCluster3);
        result.put("psCluster4",psCluster4);
        result.put("seasonMaxUb1",seasonMaxUb1);
        result.put("seasonMaxUb2",seasonMaxUb2);
        result.put("seasonMaxUb3",seasonMaxUb3);
        result.put("seasonMaxUb4",seasonMaxUb4);
        result.put("seasonClusterUb1",seasonClusterUb1);
        result.put("seasonClusterUb2",seasonClusterUb2);
        result.put("seasonClusterUb3",seasonClusterUb3);
        result.put("seasonClusterUb4",seasonClusterUb4);
        result.put("psClusterUb1",psClusterUb1);
        result.put("psClusterUb2",psClusterUb2);
        result.put("psClusterUb3",psClusterUb3);
        result.put("psClusterUb4",psClusterUb4);
        result.put("cap1",cap1);
        result.put("cap2",cap2);
        result.put("cap3",cap3);
        result.put("cap4",cap4);
        return result;
    }

    public List dataImportList() {
        ImportData importData1 = new ImportData("新建变电站","溪南G134线","2021.11.11 15:11:23");
        ImportData importData2 = new ImportData("新建变电站","溪南G134线","2021.11.12 13:32:32");
        ImportData importData3 = new ImportData("新建变电站","溪南G134线","2021.11.14 09:34:43");
        List<ImportData> result = new ArrayList<ImportData>();
        result.add(importData1);
        result.add(importData2);
        result.add(importData3);
        return result;
    }

    public List lineNameList() {
        // allFeederName为查询所有馈线名称。args[1]为存储所有馈线数据的数据库文件的路径
        // feederNames为馈线名称列表
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        List<String> feederNames = sqliteDb.queryAllFeederName(allFeederNameTable);
        return feederNames;
    }

    public Object loadPosAnalysis(String lineName, int cap, int type) {
        // loadPos为分析负荷接入位置，args[1]为馈线数据库文件夹的路径，feederDbName为馈线名称，cimFile为.xml文件夹的路径
        // args[4]为负荷容量，args[5]为负荷特征（1为峰用电，2为谷用电，3为峰谷用电），args[6]为存储所有馈线数据的数据库文件的路径
        SqliteDb sqliteDb = new SqliteDb(feederDbFile + "\\" + feederDbName + ".db");
        String[] sourceStationNames = new String[]{sqliteDb.querySubstationName(feederDbName + substationTableName)};
        JyPowerSystem ps = new JyPowerSystem(sourceStationNames);
        try {
            ps.loadFromCimXML(new FileInputStream(new File(cimFile + "\\" + feederDbName + "单线图.sln.xml")));
        } catch (FileNotFoundException e) {
            e.printStackTrace();
        }
        ps.createActiveIslands();
        AvailCapModel availCapModel = new AvailCapModel(ps);
        availCapModel.buildPaths();
        availCapModel.createLoadPosTable(allPsDbFile, loadPosTable);
        double loadCap = cap;
        int loadType = type;
        // 生成负荷曲线
        double[] load = new double[96];
        double peakAvg = 0.75 * loadCap;
        double valleyAvg = 0.4 * loadCap;
        double pvAvg = 0.7 * loadCap;
        double minLoad = 0.2 * loadCap;
        if (loadType == 1) {
            load[0] = valleyAvg;
            for (int i = 1; i < 32; i++) {
                double r = 0.1 * (Math.random() - 0.5) * loadCap;
                load[i] = load[i - 1] + r;
                load[i] = Math.min(load[i], loadCap);
                load[i] = Math.max(load[i], minLoad);
            }
            load[32] = peakAvg;
            for (int i = 33; i < 88; i++) {
                double r = 0.1 * (Math.random() - 0.5) * loadCap;
                load[i] = load[i - 1] + r;
                load[i] = Math.min(load[i], loadCap);
                load[i] = Math.max(load[i], minLoad);
            }
            load[88] = valleyAvg;
            for (int i = 89; i < 96; i++) {
                double r = 0.1 * (Math.random() - 0.5) * loadCap;
                load[i] = load[i - 1] + r;
                load[i] = Math.min(load[i], loadCap);
                load[i] = Math.max(load[i], minLoad);
            }
        } else if (loadType == 2) {
            load[0] = peakAvg;
            for (int i = 1; i < 32; i++) {
                double r = 0.1 * (Math.random() - 0.5) * loadCap;
                load[i] = load[i - 1] + r;
                load[i] = Math.min(load[i], loadCap);
                load[i] = Math.max(load[i], minLoad);
            }
            load[32] = valleyAvg;
            for (int i = 33; i < 88; i++) {
                double r = 0.1 * (Math.random() - 0.5) * loadCap;
                load[i] = load[i - 1] + r;
                load[i] = Math.min(load[i], loadCap);
                load[i] = Math.max(load[i], minLoad);
            }
            load[88] = peakAvg;
            for (int i = 89; i < 96; i++) {
                double r = 0.1 * (Math.random() - 0.5) * loadCap;
                load[i] = load[i - 1] + r;
                load[i] = Math.min(load[i], loadCap);
                load[i] = Math.max(load[i], minLoad);
            }
        } else if (loadType == 3) {
            load[0] = pvAvg;
            for (int i = 1; i < 96; i++) {
                double r = 0.1 * (Math.random() - 0.5) * loadCap;
                load[i] = load[i - 1] + r;
                load[i] = Math.min(load[i], loadCap);
                load[i] = Math.max(load[i], minLoad);
            }
        }
        for (int i = 0; i < 96; i++) {
            System.out.println(load[i] + ",");
        }
        // 负荷接入分析
        LoadPos loadPos = availCapModel.loadPosOpt(load, feederDbName + switchTableName,
                feederDbName + availCapTableName, feederDbFile + "\\" + feederDbName + ".db",
                feederDbName + substationTableName, feederDbName + feederTableName,
                feederDbName + oneLineParamTableName, feederDbName + switchTableName + HistoryData.seasonClusterTable,
                feederDbName + swToTfTableName, feederDbName + tfParamTableName,
                feederDbName + transformerTableName + HistoryData.tfAvailCapTable,
                feederDbName + transformerTableName + HistoryData.seasonClusterTable,
                feederDbName + transformerTableName + HistoryData.minITable, allPsDbFile, loadPosTable);

        return loadPos;
    }

    public Map<String,Object> lowMarginAnalysis() {
        // feederWarnDev为查询预警元件颜色。args[1]为馈线数据库文件夹路径，feederDbName为查询的馈线名称
        // yellowTfs为显示黄色的公变列表，redTfs为显示红色的公变列表，yellowLines为显示黄色的线路列表，redLines为显示红色的线路列表，
        // 属性loadState为1表示黄色，为2表示红色，属性mRID为设备mRID
        SqliteDb sqliteDb = new SqliteDb(feederDbFile + "\\" + feederDbName + ".db");
        List<WarnTf> warnTfs = sqliteDb.queryWarnTf(feederDbName + tfWarnTableName);
        List<WarnLine> warnLines = sqliteDb.queryWarnLine(feederDbName + lineWarnTableName);
        List<WarnTf> yellowTfs = new LinkedList<>();    // 显示黄色的公变
        List<WarnTf> redTfs = new LinkedList<>();   // 显示红色的公变
        List<WarnLine> yellowLines = new LinkedList<>();   // 显示黄色的线路
        List<WarnLine> redLines = new LinkedList<>();    // 显示红色的线路
        for (WarnTf warnTf : warnTfs) {
            if (warnTf.getLoadState() == 1)
                yellowTfs.add(warnTf);
            else
                redTfs.add(warnTf);
        }
        for (WarnLine warnLine : warnLines) {
            if (warnLine.getLoadState() == 1)
                yellowLines.add(warnLine);
            else
                redLines.add(warnLine);
        }
        Map<String,Object> result = new HashMap<>();
        result.put("yellowTfs",yellowTfs);
        result.put("redTfs",redTfs);
        result.put("yellowLines",yellowLines);
        result.put("redLines",redLines);
        return result;
    }
}
