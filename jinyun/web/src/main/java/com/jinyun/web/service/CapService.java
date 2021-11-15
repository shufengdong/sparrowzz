package com.jinyun.web.service;

import com.jinyun.cap.*;
import org.springframework.stereotype.Service;

import java.util.HashMap;
import java.util.LinkedList;
import java.util.List;
import java.util.Map;

/**
 * @author jinbin
 * @date 2018-07-08 20:52
 */
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
            if(count == 3) break;
            if(result.containsKey(tfUb.getDevName()))
                continue;
            result.put(tfUb.getDevName(),tfUb.getUb());
            count ++;
        }
        return result;
    }

    public Map<String, Object> transformerUnbalanceList() {
        SqliteDb allPsDb = new SqliteDb(allPsDbFile);
        List<String> feeders = allPsDb.queryAllFeederName(allFeederNameTable);  // 查询馈线名称
        List<TfUb> tfUbs = new LinkedList<>();
        for (String feeder : feeders) {
            SqliteDb sqliteDb = new SqliteDb(feederDbFile + "\\" + feeder + ".db");
            tfUbs.addAll(sqliteDb.queryTfMonthUb(feeder + tfMonthUbTableName));
        }
        Map<String,Object> result = new HashMap<>();
        for(TfUb tfUb:tfUbs){
            result.put(tfUb.getmRID(),tfUb);
        }
        return result;
    }

    public List hardLineList() {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        List<WarnLine> hardLine = sqliteDb.queryWarnLine(allPsLineWarnTable, 1);
        return hardLine;
    }

    public List hardTransformerList() {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        List<WarnLine> overLine = sqliteDb.queryWarnLine(allPsLineWarnTable, 2);
        return overLine;
    }

    public List overLineList() {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        List<WarnTf> hardTf = sqliteDb.queryWarnTf(allPsTfWarnTable, 1);
        return hardTf;
    }

    public List overTransformerList() {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        List<WarnTf> overTf = sqliteDb.queryWarnTf(allPsTfWarnTable, 2);
        return overTf;
    }

    public List linePassRate() {
        SqliteDb sqliteDb = new SqliteDb(feederDbFile + "\\" + feederDbName + ".db");
        List<LinePassRate> linePassRates = sqliteDb.queryLinePassRate(feederDbName + linePassRateTableName);
        return linePassRates;
    }

    public Object transformerOverLoadRate() {
        SqliteDb allPsDb = new SqliteDb(allPsDbFile);
        List<String> feeders = allPsDb.queryAllFeederName(allFeederNameTable);  // 查询馈线名称
        List<TfOverLoadRate> tfOverLoadRates = new LinkedList<>();
        for (String feeder : feeders) {
            SqliteDb sqliteDb = new SqliteDb(feederDbFile + "\\" + feeder + ".db");
            tfOverLoadRates.addAll(sqliteDb.queryTfOverLoadRate(feeder + tfOverLoadRateTableName));
        }
        return tfOverLoadRates.get(0); //todo 先取第一个
    }

    public List maxMinAvailCap() {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        MaxMinAvailCap maxFeeder = sqliteDb.queryMaxMinAvailCap(maxMinAvailCapTable, 1);
        MaxMinAvailCap minFeeder = sqliteDb.queryMaxMinAvailCap(maxMinAvailCapTable, 2);
        List result = new LinkedList<>();
        result.add(maxFeeder);
        result.add(minFeeder);
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
        double lineRatedI = sqliteDb.queryOneLineParam(feederDbName + oneLineParamTableName, mRID);
        double[] lineRatedICv = new double[96];
        for (int i = 0; i < 96; i++) {
            lineRatedICv[i] = lineRatedI;
        }
        double[] seasonCluster2 = sqliteDb.querySeasonSwitchI(feederDbName + switchTableName + HistoryData.seasonClusterTable, mRID, 2, 96);
        double[] seasonMax2 = sqliteDb.querySeasonSwitchI(feederDbName + switchTableName + HistoryData.seasonTable, mRID, 2, 96);
        double[] cap2 = sqliteDb.queryAvailCap(feederDbName + availCapTableName, mRID, 2, 96);
        Map<String,Object> result = new HashMap<>();
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
        result.put("psCluster2",psCluster2);
        result.put("seasonMax2",seasonMax2);
        result.put("cap2",cap2);
        return result;
    }

    public List loadPosList() {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        List<LoadPosSeason> loadPosSeason = sqliteDb.queryLoadPosSeason(loadPosTable, 1);
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
}
