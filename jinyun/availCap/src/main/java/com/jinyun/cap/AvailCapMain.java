package com.jinyun.cap;//package availCap;

import zju.devmodel.MapObject;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileNotFoundException;
import java.util.LinkedList;
import java.util.List;
import java.util.Map;

public class AvailCapMain {

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

    public static void main(String[] args) {
        // args[0]选择调用的方法
        switch (args[0]) {
            case "parseData":
                // parseData为解析数据文件和存库操作，args[1]为馈线数据库文件夹的路径，args[2]为馈线名称，args[3]为线路电流历史数据文件夹的路径
                // args[4]为开关历史数据文件夹的路径，args[5]为线路参数.csv格式文件数据，args[6]为公变历史数据文件夹的路径，
                // args[7]为公变参数文件的路径，args[8]为存储所有馈线数据的数据库文件的路径，args[9]为变电站名称，args[10]为.xml文件的路径
                // 馈线名称和id
                JyPowerSystem ps = new JyPowerSystem();
                try {
                    ps.loadFromCimXML(new FileInputStream(new File(args[10])));
                } catch (FileNotFoundException e) {
                    e.printStackTrace();
                }
                List<MapObject> feederObjs = ps.getResourceByClassId("FEEDER");
                String feederMRID = null;
                String feederName = null;
                for (MapObject feeder: feederObjs) {
                    if (feeder.getProperty("ISCURRENTFEEDER").equals("true")) {
                        feederMRID = feeder.getProperty("MRID");
                        feederName = feeder.getProperty("NAME");
                    }
                }
                HistoryData.createLineCurrentTable(args[1] + "\\" + args[2] + ".db", args[2] + lineITableName);
                HistoryData.parseLineCurrent(args[2] + lineITableName, args[3], args[1] + "\\" + args[2] + ".db");
                HistoryData.createSwitchTable(args[1] + "\\" + args[2] + ".db", args[2] + switchTableName);
                HistoryData.parseSwitch(args[2] + switchTableName, args[4], args[1] + "\\" + args[2] + ".db");
                HistoryData.createLineParamTable(args[1] + "\\" + args[2] + ".db", args[2] + lineParamTableName);
                HistoryData.parseLineParam(args[2] + lineParamTableName, args[5], args[1] + "\\" + args[2] + ".db");
                HistoryData.createTFTable(args[1] + "\\" + args[2] + ".db", args[2] + transformerTableName);
                HistoryData.parseTF(args[2] + transformerTableName, args[6], args[1] + "\\" + args[2] + ".db");
                HistoryData.createTFParamTable(args[1] + "\\" + args[2] + ".db", args[2] + tfParamTableName);
                HistoryData.parseTFParam(args[2] + tfParamTableName, args[7], args[1] + "\\" + args[2] + ".db");
                HistoryData.parseTFMRID(args[2] + tfParamTableName,
                        args[2] + transformerTableName, args[1] + "\\" + args[2] + ".db");
                HistoryData historyData = new HistoryData();
                // 历史数据处理
                historyData.lineCurrentDataAnalysis(args[2] + lineITableName, args[1] + "\\" + args[2] + ".db");
                historyData.switchDataAnalysis(args[2] + switchTableName, args[1] + "\\" + args[2] + ".db");
                historyData.switchDataSeasonCluster(args[2] + switchTableName, args[1] + "\\" + args[2] + ".db");
                historyData.switchDataPsCluster(args[2] + switchTableName, args[1] + "\\" + args[2] + ".db");
                historyData.tfDataAnalysis(args[2] + transformerTableName, args[1] + "\\" + args[2] + ".db");
                historyData.tfDataSeasonCluster(args[2] + transformerTableName, args[1] + "\\" + args[2] + ".db");
                historyData.tfDataPsCluster(args[2] + transformerTableName, args[1] + "\\" + args[2] + ".db");
                historyData.tfAvailCap(args[2] + transformerTableName + HistoryData.seasonTable,
                        args[2] + tfParamTableName, args[2] + transformerTableName, args[1] + "\\" + args[2] + ".db");
                // 存储馈线名称
                historyData.createFeederNameTable(args[1] + "\\" + args[2] + ".db", args[2] + feederTableName);
                historyData.saveFeederName(args[2] + feederTableName, args[1] + "\\" + args[2] + ".db", args[2]);
                // 存储所有馈线名称
                AvailCapModel availCapModel = new AvailCapModel();
                availCapModel.createAllFeederNameTable(args[8], allFeederNameTable);
                availCapModel.saveAllFeederNameTable(args[8], allFeederNameTable, feederMRID, feederName);
                // 存储变电站名称
                availCapModel.createSubStationNameTable(args[1] + "\\" + args[2] + ".db", args[2] + substationTableName);
                availCapModel.saveSubstationName(args[2] + substationTableName,
                        args[1] + "\\" + args[2] + ".db", args[9]);
                break;
            case "calAvailCap": {
                // calAvailCap为计算可接入容量操作，args[1]为馈线数据库文件夹的路径，args[2]为馈线名称，args[3]为.xml文件的路径
                SqliteDb sqliteDb = new SqliteDb(args[1] + "\\" + args[2] + ".db");
                String[] sourceStationNames = new String[]{sqliteDb.querySubstationName(args[2] + substationTableName)};
                ps = new JyPowerSystem(sourceStationNames);
                try {
                    ps.loadFromCimXML(new FileInputStream(new File(args[3])));
                } catch (FileNotFoundException e) {
                    e.printStackTrace();
                }
                ps.createActiveIslands();
                availCapModel = new AvailCapModel(ps);
                availCapModel.buildPaths();
//                availCapModel.setEdgeLimI(args[2] + lineParamTableName, args[1] + "\\" + args[2] + ".db");
                // 设置单线限额并存库
                availCapModel.createOneLineParamTable(args[1] + "\\" + args[2] + ".db", args[2] + oneLineParamTableName);
                availCapModel.setEdgeLimI(args[2] + lineParamTableName,
                        args[1] + "\\" + args[2] + ".db", args[2] + oneLineParamTableName);
                availCapModel.setEdgeAvailCap(args[2] + lineITableName + HistoryData.seasonTable,
                        args[2] + switchTableName + HistoryData.seasonTable, args[1] + "\\" + args[2] + ".db");
                availCapModel.createAvailCapTable(args[1] + "\\" + args[2] + ".db", args[2] + availCapTableName);
                availCapModel.calAvailCap(args[2] + availCapTableName,
                        args[2] + switchTableName, args[1] + "\\" + args[2] + ".db");
                break;
            }
            case "warnDev": {
                // warnDev为设备预警分析。args[1]为馈线数据库文件夹的路径，args[2]为馈线名称，args[3]为.xml文件的路径
                SqliteDb sqliteDb = new SqliteDb(args[1] + "\\" + args[2] + ".db");
                String[] sourceStationNames = new String[]{sqliteDb.querySubstationName(args[2] + substationTableName)};
                ps = new JyPowerSystem(sourceStationNames);
                try {
                    ps.loadFromCimXML(new FileInputStream(new File(args[3])));
                } catch (FileNotFoundException e) {
                    e.printStackTrace();
                }
                ps.createActiveIslands();
                availCapModel = new AvailCapModel(ps);
                availCapModel.buildPaths();

                availCapModel.createSwToLineTable(args[1] + "\\" + args[2] + ".db", args[2] + swToLineTableName);
                availCapModel.createSwToTfTable(args[1] + "\\" + args[2] + ".db", args[2] + swToTfTableName);
                availCapModel.createTfToLineTable(args[1] + "\\" + args[2] + ".db", args[2] + tfToLineTableName);
                availCapModel.createLineWarnTable(args[1] + "\\" + args[2] + ".db", args[2] + lineWarnTableName);
                availCapModel.createTfWarnTable(args[1] + "\\" + args[2] + ".db", args[2] + tfWarnTableName);
                availCapModel.switchToLine(args[2] + switchTableName,
                        args[1] + "\\" + args[2] + ".db", args[2] + swToLineTableName);
                availCapModel.switchToTf(args[2] + switchTableName, args[1] + "\\" + args[2] + ".db",
                        args[2] + swToTfTableName, args[2] + tfToLineTableName);
                availCapModel.warnDevAnalysis(args[1] + "\\" + args[2] + ".db", args[2] + substationTableName,
                        args[2] + tfWarnTableName, args[2] + transformerTableName,
                        args[2] + tfParamTableName, args[2] + transformerTableName + HistoryData.seasonTable,
                        args[2] + tfToLineTableName, args[2] + switchTableName,args[2] + swToLineTableName,
                        args[2] + oneLineParamTableName, args[2] + switchTableName + HistoryData.seasonTable,
                        args[2] + lineWarnTableName);
//                List<WarnTf> warnTfs = sqliteDb.queryWarnTf(args[2] + tfWarnTableName);
//                List<WarnLine> warnLines = sqliteDb.queryWarnLine(args[2] + lineWarnTableName);
//                System.out.println(warnTfs.size() + ", " + warnLines.size());
                break;
            }
            case "loadPos": {
                // loadPos为分析负荷接入位置，args[1]为馈线数据库文件夹的路径，args[2]为馈线名称，args[3]为.xml文件夹的路径
                // args[4]为负荷容量，args[5]为负荷特征（1为峰用电，2为谷用电，3为峰谷用电），args[6]为存储所有馈线数据的数据库文件的路径
                SqliteDb sqliteDb = new SqliteDb(args[1] + "\\" + args[2] + ".db");
                String[] sourceStationNames = new String[]{sqliteDb.querySubstationName(args[2] + substationTableName)};
                ps = new JyPowerSystem(sourceStationNames);
                try {
                    ps.loadFromCimXML(new FileInputStream(new File(args[3] + "\\" + args[2] + "单线图.sln.xml")));
                } catch (FileNotFoundException e) {
                    e.printStackTrace();
                }
                ps.createActiveIslands();
                availCapModel = new AvailCapModel(ps);
                availCapModel.buildPaths();
                availCapModel.createLoadPosTable(args[6], loadPosTable);
                double loadCap = Double.parseDouble(args[4]);
                int loadType = Integer.parseInt(args[5]);
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
                    for (int i = 1; i < 32; i++) {
                        double r = 0.08 * (Math.random() - 0.5) * loadCap;
                        load[i] = load[i - 1] + r;
                        load[i] = Math.min(load[i], loadCap);
                        load[i] = Math.max(load[i], minLoad);
                    }
                    load[32] = pvAvg;
                    for (int i = 33; i < 88; i++) {
                        double r = 0.05 * (Math.random() - 0.5) * loadCap;
                        load[i] = load[i - 1] + r;
                        load[i] = Math.min(load[i], loadCap);
                        load[i] = Math.max(load[i], minLoad);
                    }
                    load[88] = pvAvg;
                    for (int i = 89; i < 96; i++) {
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
                LoadPos loadPos = availCapModel.loadPosOpt(load, args[2] + switchTableName,
                        args[2] + availCapTableName, args[1] + "\\" + args[2] + ".db",
                        args[2] + substationTableName, args[2] + feederTableName,
                        args[2] + oneLineParamTableName, args[2] + switchTableName + HistoryData.seasonClusterTable,
                        args[2] + swToTfTableName, args[2] + tfParamTableName,
                        args[2] + transformerTableName + HistoryData.tfAvailCapTable,
                        args[2] + transformerTableName + HistoryData.seasonClusterTable,
                        args[2] + transformerTableName + HistoryData.minITable, args[6], loadPosTable);
                break;
            }
            case "mouseOverSw": {
                // mouseOverSw为鼠标放置在开关上，查询线路限额，最大电流，可开放容量。args[1]为馈线数据库文件夹的路径，args[2]为馈线名称，args[3]为线路mRID
                SqliteDb sqliteDb = new SqliteDb(args[1] + "\\" + args[2] + ".db");
                double lineRatedI = sqliteDb.queryOneLineParam(args[2] + oneLineParamTableName, args[3]);
                double lineMaxI = sqliteDb.queryMaxSwitchI(args[2] + switchTableName + HistoryData.seasonTable, args[3], -1);
                double[] cap1 = sqliteDb.queryAvailCap(args[2] + availCapTableName, args[3], 1, 96);
                double[] cap2 = sqliteDb.queryAvailCap(args[2] + availCapTableName, args[3], 2, 96);
                double[] cap3 = sqliteDb.queryAvailCap(args[2] + availCapTableName, args[3], 3, 96);
                double[] cap4 = sqliteDb.queryAvailCap(args[2] + availCapTableName, args[3], 4, 96);
                break;
            }
            case "mouseOverTF": {
                // mouseOverTF为鼠标放置在公变上，查询公变容量，最大负荷，三相不平衡度。args[1]为馈线数据库文件夹的路径，args[2]为馈线名称，args[3]为公变mRID
                SqliteDb sqliteDb = new SqliteDb(args[1] + "\\" + args[2] + ".db");
                double tFRatedCap = sqliteDb.queryTFCap(args[2] + tfParamTableName, args[3]);
                double tFMaxP = sqliteDb.queryMaxTFP(args[2] + transformerTableName + HistoryData.seasonTable, args[3], -1);
                double[] ub = sqliteDb.queryTFUb(args[2] + transformerTableName + HistoryData.unbalanceTable, args[3], -1);
                // 低压负荷接入相别
                String phase = sqliteDb.queryMinIPhase(args[2] + transformerTableName + HistoryData.minITable, args[3], -1);
                System.out.println(tFRatedCap + "        " + tFMaxP + "     " + ub[0] + "    " + ub[1] + "     " + phase);
                break;
            }
            case "mouseClickSw": {
                // mouseClickSw为鼠标点击开关上，查询线路限额，最大电流，可开放容量。args[1]为馈线数据库文件夹的路径，args[2]为馈线名称，args[3]为开关mRID
                SqliteDb sqliteDb = new SqliteDb(args[1] + "\\" + args[2] + ".db");
                double lineRatedI = sqliteDb.queryOneLineParam(args[2] + oneLineParamTableName, args[3]);
                double lineMaxI = sqliteDb.queryMaxSwitchI(args[2] + switchTableName + HistoryData.seasonTable, args[3], -1);
                double lineAvgI = sqliteDb.queryMaxSwitchI(args[2] + switchTableName + HistoryData.seasonTable, args[3], -2);
                double[] cap1 = sqliteDb.queryAvailCap(args[2] + availCapTableName, args[3], 1, 96);
                double[] cap2 = sqliteDb.queryAvailCap(args[2] + availCapTableName, args[3], 2, 96);
                double[] cap3 = sqliteDb.queryAvailCap(args[2] + availCapTableName, args[3], 3, 96);
                double[] cap4 = sqliteDb.queryAvailCap(args[2] + availCapTableName, args[3], 4, 96);
                double[] seasonCluster1 = sqliteDb.querySeasonSwitchI(args[2] + switchTableName + HistoryData.seasonClusterTable, args[3], 1, 96);
                double[] seasonCluster2 = sqliteDb.querySeasonSwitchI(args[2] + switchTableName + HistoryData.seasonClusterTable, args[3], 2, 96);
                double[] seasonCluster3 = sqliteDb.querySeasonSwitchI(args[2] + switchTableName + HistoryData.seasonClusterTable, args[3], 3, 96);
                double[] seasonCluster4 = sqliteDb.querySeasonSwitchI(args[2] + switchTableName + HistoryData.seasonClusterTable, args[3], 4, 96);
                double[] psCluster1 = sqliteDb.querySeasonSwitchI(args[2] + switchTableName + HistoryData.psClusterTable, args[3], 1, 96);
                double[] psCluster2 = sqliteDb.querySeasonSwitchI(args[2] + switchTableName + HistoryData.psClusterTable, args[3], 2, 96);
                double[] psCluster3 = sqliteDb.querySeasonSwitchI(args[2] + switchTableName + HistoryData.psClusterTable, args[3], 3, 96);
                double[] psCluster4 = sqliteDb.querySeasonSwitchI(args[2] + switchTableName + HistoryData.psClusterTable, args[3], 4, 96);
                double[] seasonMax1 = sqliteDb.querySeasonSwitchI(args[2] + switchTableName + HistoryData.seasonTable, args[3], 1, 96);
                double[] seasonMax2 = sqliteDb.querySeasonSwitchI(args[2] + switchTableName + HistoryData.seasonTable, args[3], 2, 96);
                double[] seasonMax3 = sqliteDb.querySeasonSwitchI(args[2] + switchTableName + HistoryData.seasonTable, args[3], 3, 96);
                double[] seasonMax4 = sqliteDb.querySeasonSwitchI(args[2] + switchTableName + HistoryData.seasonTable, args[3], 4, 96);
                for (int i = 0; i < 96; i++) {
                    System.out.println(lineRatedI + "," + seasonCluster1[i] + "," + seasonMax1[i] + "," + seasonCluster2[i] + "," + seasonMax2[i] + "," +
                            seasonCluster3[i] + "," + seasonMax3[i] + "," + seasonCluster4[i] + "," + seasonMax4[i]);
                }
                System.out.println();
                for (int i = 0; i < 96; i++) {
                    System.out.println(lineRatedI + "," + psCluster1[i] + "," + seasonMax1[i] + "," + psCluster2[i] + "," + seasonMax2[i] + "," +
                            psCluster3[i] + "," + seasonMax3[i] + "," + psCluster4[i] + "," + seasonMax4[i]);
                }
                break;
            }
            case "mouseClickTF": {
                // mouseClickTF为鼠标点击公变上，查询公变容量，最大负荷，三相不平衡度。args[1]为馈线数据库文件夹的路径，args[2]为馈线名称，args[3]为公变mRID
                SqliteDb sqliteDb = new SqliteDb(args[1] + "\\" + args[2] + ".db");
                double tFRatedCap = sqliteDb.queryTFCap(args[2] + tfParamTableName, args[3]);
                double tFMaxI = sqliteDb.queryMaxTFP(args[2] + transformerTableName + HistoryData.seasonTable, args[3], -2);
                double[] ub = sqliteDb.queryTFUb(args[2] + transformerTableName + HistoryData.unbalanceTable, args[3], -1);
                // 负荷
                double[] seasonMax1 = sqliteDb.querySeasonTFP(args[2] + transformerTableName + HistoryData.seasonTable, args[3], 1, 96);
                double[] seasonMax2 = sqliteDb.querySeasonTFP(args[2] + transformerTableName + HistoryData.seasonTable, args[3], 2, 96);
                double[] seasonMax3 = sqliteDb.querySeasonTFP(args[2] + transformerTableName + HistoryData.seasonTable, args[3], 3, 96);
                double[] seasonMax4 = sqliteDb.querySeasonTFP(args[2] + transformerTableName + HistoryData.seasonTable, args[3], 4, 96);
                double[] seasonCluster1 = sqliteDb.querySeasonTFP(args[2] + transformerTableName + HistoryData.seasonClusterTable, args[3], 1, 96);
                double[] seasonCluster2 = sqliteDb.querySeasonTFP(args[2] + transformerTableName + HistoryData.seasonClusterTable, args[3], 2, 96);
                double[] seasonCluster3 = sqliteDb.querySeasonTFP(args[2] + transformerTableName + HistoryData.seasonClusterTable, args[3], 3, 96);
                double[] seasonCluster4 = sqliteDb.querySeasonTFP(args[2] + transformerTableName + HistoryData.seasonClusterTable, args[3], 4, 96);
                double[] psCluster1 = sqliteDb.querySeasonTFP(args[2] + transformerTableName + HistoryData.psClusterTable, args[3], 1, 96);
                double[] psCluster2 = sqliteDb.querySeasonTFP(args[2] + transformerTableName + HistoryData.psClusterTable, args[3], 2, 96);
                double[] psCluster3 = sqliteDb.querySeasonTFP(args[2] + transformerTableName + HistoryData.psClusterTable, args[3], 3, 96);
                double[] psCluster4 = sqliteDb.querySeasonTFP(args[2] + transformerTableName + HistoryData.psClusterTable, args[3], 4, 96);
                // 三相不平衡度
                double[] seasonMaxUb1 = sqliteDb.querySeasonTFUb(args[2] + transformerTableName + HistoryData.ubMaxTable, args[3], 1, 96);
                double[] seasonMaxUb2 = sqliteDb.querySeasonTFUb(args[2] + transformerTableName + HistoryData.ubMaxTable, args[3], 2, 96);
                double[] seasonMaxUb3 = sqliteDb.querySeasonTFUb(args[2] + transformerTableName + HistoryData.ubMaxTable, args[3], 3, 96);
                double[] seasonMaxUb4 = sqliteDb.querySeasonTFUb(args[2] + transformerTableName + HistoryData.ubMaxTable, args[3], 4, 96);
                double[] seasonClusterUb1 = sqliteDb.querySeasonTFUb(args[2] + transformerTableName + HistoryData.seasonClusterTable, args[3], 1, 96);
                double[] seasonClusterUb2 = sqliteDb.querySeasonTFUb(args[2] + transformerTableName + HistoryData.seasonClusterTable, args[3], 2, 96);
                double[] seasonClusterUb3 = sqliteDb.querySeasonTFUb(args[2] + transformerTableName + HistoryData.seasonClusterTable, args[3], 3, 96);
                double[] seasonClusterUb4 = sqliteDb.querySeasonTFUb(args[2] + transformerTableName + HistoryData.seasonClusterTable, args[3], 4, 96);
                double[] psClusterUb1 = sqliteDb.querySeasonTFUb(args[2] + transformerTableName + HistoryData.psClusterTable, args[3], 1, 96);
                double[] psClusterUb2 = sqliteDb.querySeasonTFUb(args[2] + transformerTableName + HistoryData.psClusterTable, args[3], 2, 96);
                double[] psClusterUb3 = sqliteDb.querySeasonTFUb(args[2] + transformerTableName + HistoryData.psClusterTable, args[3], 3, 96);
                double[] psClusterUb4 = sqliteDb.querySeasonTFUb(args[2] + transformerTableName + HistoryData.psClusterTable, args[3], 4, 96);
                // 公变可开放容量
                double[] cap1 = sqliteDb.queryAvailCap(args[2] + transformerTableName + HistoryData.tfAvailCapTable, args[3], 1, 96);
                double[] cap2 = sqliteDb.queryAvailCap(args[2] + transformerTableName + HistoryData.tfAvailCapTable, args[3], 2, 96);
                double[] cap3 = sqliteDb.queryAvailCap(args[2] + transformerTableName + HistoryData.tfAvailCapTable, args[3], 3, 96);
                double[] cap4 = sqliteDb.queryAvailCap(args[2] + transformerTableName + HistoryData.tfAvailCapTable, args[3], 4, 96);
                System.out.println(tFRatedCap + "        " + tFMaxI + "     " + ub[0] + "    " + ub[1]);
                break;
            }
            case "calAllPsWarnDev": {
                // calAllPsWarnDev为统计所有馈线设备预警数量。args[1]为存储所有馈线数据的数据库文件的路径，args[2]为馈线数据库文件夹路径
                availCapModel = new AvailCapModel();
                availCapModel.createAllPsWarnTable(args[1], allPsWarnTable);
                availCapModel.createAllPsLineWarnTable(args[1], allPsLineWarnTable);
                availCapModel.createAllPsTfWarnTable(args[1], allPsTfWarnTable);
                availCapModel.allPsWarnDev(args[1], allFeederNameTable, args[2], allPsWarnTable, allPsLineWarnTable, allPsTfWarnTable);
                System.out.println();
                break;
            }
            case "calTfUb": {
                // tfUb为公变月三相不平衡度分析。args[1]为馈线数据库文件夹路径，args[2]为馈线名称
                historyData = new HistoryData();
                HistoryData.createTfMonthUbTable(args[1] + "\\" + args[2] + ".db", args[2] + tfMonthUbTableName);
                historyData.tfMonthUb(args[2] + transformerTableName, args[1] + "\\" + args[2] + ".db",
                        args[2] + tfMonthUbTableName, args[2] + substationTableName,
                        args[2] + tfParamTableName, args[2] + transformerTableName + HistoryData.seasonTable,
                        args[2] + tfToLineTableName);
                break;
            }
            case "calLinePassRate": {
                // linePassRate为线路合格率分析。args[1]为馈线数据库文件夹路径，args[2]为馈线名称
                historyData = new HistoryData();
                historyData.createLinePassRateTable(args[1] + "\\" + args[2] + ".db", args[2] + linePassRateTableName);
                historyData.linePassRate(args[1] + "\\" + args[2] + ".db", args[2] + feederTableName,
                        args[2] + substationTableName, args[2] + switchTableName,
                        args[2] + swToLineTableName, args[2] + oneLineParamTableName,
                        args[2] + linePassRateTableName);
                break;
            }
            case "calTfOverLoadRate": {
                // tfOverLoadRate为公变越限分析。args[1]为馈线数据库文件夹路径，args[2]为馈线名称
                historyData = new HistoryData();
                historyData.createTfOverLoadTable(args[1] + "\\" + args[2] + ".db", args[2] + tfOverLoadRateTableName);
                historyData.tfOverLoad(args[1] + "\\" + args[2] + ".db", args[2] + feederTableName,
                        args[2] + substationTableName, args[2] + tfOverLoadRateTableName,
                        args[2] + transformerTableName, args[2] + tfParamTableName,
                        args[2] + transformerTableName + HistoryData.seasonTable, args[2] + tfToLineTableName);
                break;
            }
            case "calMaxMinAvailCap": {
                // calMaxMinAvailCap为分析可开放容量最大和最小两条线路。args[1]为存储所有馈线数据的数据库文件的路径，args[2]为馈线数据库文件夹路径
                availCapModel = new AvailCapModel();
                availCapModel.createMaxMinAvailCap(args[1], maxMinAvailCapTable);
                availCapModel.maxMinAvailCap(args[1], allFeederNameTable, args[2], maxMinAvailCapTable);
                System.out.println();
                break;
            }
            case "allFeederName": {
                // allFeederName为查询所有馈线名称。args[1]为存储所有馈线数据的数据库文件的路径
                // feederNames为馈线名称列表
                SqliteDb sqliteDb = new SqliteDb(args[1]);
                List<String> feederNames = sqliteDb.queryAllFeederName(allFeederNameTable);
                System.out.println(feederNames.size());
                break;
            }
            case "allFeederNameId": {
                // allFeederNameId为查询所有馈线名称和id对应关系。args[1]为存储所有馈线数据的数据库文件的路径
                // feederIdToName为馈线id到名称的映射，feederNameToId为馈线名称到id的映射
                SqliteDb sqliteDb = new SqliteDb(args[1]);
                FeederNameId feederNameId = sqliteDb.queryAllFeederNameId(allFeederNameTable);
                Map<String, String> feederIdToName = feederNameId.getFeederIdToName();
                Map<String, String> feederNameToId = feederNameId.getFeederNameToId();
                System.out.println(feederIdToName.size() + "," + feederNameToId.size());
                break;
            }
            case "allPsWarnDev": {
                // allPsWarnDev为查询所有馈线设备预警数量。args[1]为存储所有馈线数据的数据库文件的路径
                // hardLineNum为重载线路数量，overLineNum为超载线路数量，hardTfNum为重载配变数量，overTfNum为超载配变数量
                SqliteDb sqliteDb = new SqliteDb(args[1]);
                int hardLineNum = sqliteDb.queryWarnDevNum(allPsWarnTable, 1, 1);
                int overLineNum = sqliteDb.queryWarnDevNum(allPsWarnTable, 2, 1);
                int hardTfNum = sqliteDb.queryWarnDevNum(allPsWarnTable, 1, 2);
                int overTfNum = sqliteDb.queryWarnDevNum(allPsWarnTable, 2, 2);
                System.out.println(hardLineNum + " " + overLineNum + " " + hardTfNum + " " + overTfNum);
                break;
            }
            case "allPsHardLine": {
                // allPsHardLine为查询所有馈线重载线路。args[1]为存储所有馈线数据的数据库文件的路径
                // hardLine为重载线路列表数据，属性devName;substation;maxI;switchName;ratedCurrent对应界面表格中
                // 线路名、所属变电站、最大电流、智能开关、限额
                SqliteDb sqliteDb = new SqliteDb(args[1]);
                List<WarnLine> hardLine = sqliteDb.queryWarnLine(allPsLineWarnTable, 1,-1,-1);
                System.out.println(hardLine.size());
                break;
            }
            case "allPsOverLine": {
                // allPsHardLine为查询所有馈线超载线路。args[1]为存储所有馈线数据的数据库文件的路径
                // overLine为超载线路列表数据，属性devName;substation;maxI;switchName;ratedCurrent对应界面表格中
                // 线路名、所属变电站、最大电流、智能开关、限额
                SqliteDb sqliteDb = new SqliteDb(args[1]);
                List<WarnLine> overLine = sqliteDb.queryWarnLine(allPsLineWarnTable, 2,-1,-1);
                System.out.println(overLine.size());
                break;
            }
            case "allPsHardTf": {
                // allPsHardTf为查询所有馈线重载公变。args[1]为存储所有馈线数据的数据库文件的路径
                // hardTf为重载公变列表数据，属性devName;lineName;substation;maxLoad;ratedCap对应界面表格中
                // 配变名、线路、变电站、最大负荷、容量
                SqliteDb sqliteDb = new SqliteDb(args[1]);
                List<WarnTf> hardTf = sqliteDb.queryWarnTf(allPsTfWarnTable, 1,-1,-1);
                System.out.println(hardTf.size());
                break;
            }
            case "allPsOverTf": {
                // allPsOverTf为查询所有馈线超载公变。args[1]为存储所有馈线数据的数据库文件的路径
                // overTf为超载公变列表数据，属性devName;lineName;substation;maxLoad;ratedCap对应界面表格中配变名、线路、变电站、最大负荷、容量
                SqliteDb sqliteDb = new SqliteDb(args[1]);
                List<WarnTf> overTf = sqliteDb.queryWarnTf(allPsTfWarnTable, 2,-1,-1);
                System.out.println(overTf.size());
                break;
            }
            case "tfUb": {
                // tfUb为公变月三相不平衡度查询。args[1]为存储所有馈线数据的数据库文件的路径，args[2]为馈线数据库文件夹路径
                // tfUbs公变月三相不平衡度列表，属性devName为配变名称，ub为三项不平衡度数值
                SqliteDb allPsDb = new SqliteDb(args[1]);
                List<String> feeders = allPsDb.queryAllFeederName(allFeederNameTable);  // 查询馈线名称
                List<TfUb> tfUbs = new LinkedList<>();
                for (String feeder : feeders) {
                    SqliteDb sqliteDb = new SqliteDb(args[2] + "\\" + feeder + ".db");
                    tfUbs.addAll(sqliteDb.queryTfMonthUb(feeder + tfMonthUbTableName));
                }
                System.out.println(tfUbs.size());
                break;
            }
            case "tfUbDetail": {
                // tfUb为公变月三相不平衡度点击显示具体查询。args[1]为存储所有馈线数据的数据库文件的路径，args[2]为馈线数据库文件夹路径
                // tfUbs公变月三相不平衡度列表，属性devName;lineName;substation;maxLoad;ratedCap;monthUb对应界面表格中
                // 配变名、线路、变电站、最大负荷、容量、一个月的三相不平衡度数值
                SqliteDb allPsDb = new SqliteDb(args[1]);
                List<String> feeders = allPsDb.queryAllFeederName(allFeederNameTable);  // 查询馈线名称
                List<TfUb> tfUbs = new LinkedList<>();
                for (String feeder : feeders) {
                    SqliteDb sqliteDb = new SqliteDb(args[2] + "\\" + feeder + ".db");
                    tfUbs.addAll(sqliteDb.queryTfMonthUb(feeder + tfMonthUbTableName));
                }
                System.out.println(tfUbs.size());
//                TfUb tfUb = tfUbs.get(4);
//                System.out.println(tfUb.devName);
//                for (int i = 0; i < 30; i++) {
//                    System.out.println(tfUb.monthUb[i] + ",");
//                }
                break;
            }
            case "linePassRate": {
                // linePassRate为查询线路电流合格率。args[1]为馈线数据库文件夹路径，args[2]为查询的馈线名称
                // LinePassRates为线路电流合格率结果列表，属性passRate为春夏秋冬的合格率数组
                SqliteDb sqliteDb = new SqliteDb(args[1] + "\\" + args[2] + ".db");
                List<LinePassRate> linePassRates = sqliteDb.queryLinePassRate(args[2] + linePassRateTableName);
                System.out.println(linePassRates.size());
                break;
            }
            case "tfOverLoadRate": {
                // tfOverLoadRate为查询公变越限分析结果。args[1]为存储所有馈线数据的数据库文件的路径，args[2]为馈线数据库文件夹路径
                // tfOverLoadRates为公变越限分析结果列表，属性overLoadRate为春夏秋冬的越限率数组
                SqliteDb allPsDb = new SqliteDb(args[1]);
                List<String> feeders = allPsDb.queryAllFeederName(allFeederNameTable);  // 查询馈线名称
                List<TfOverLoadRate> tfOverLoadRates = new LinkedList<>();
                for (String feeder : feeders) {
                    SqliteDb sqliteDb = new SqliteDb(args[2] + "\\" + feeder + ".db");
                    tfOverLoadRates.addAll(sqliteDb.queryTfOverLoadRate(feeder + tfOverLoadRateTableName));
                }
                System.out.println(tfOverLoadRates.size());
                break;
            }
            case "maxMinAvailCap": {
                // maxMinAvailCap为查询可开放容量最大和最小两条线路。args[1]为存储所有馈线数据的数据库文件的路径
                // maxFeeder为可开放容量最大线路，minFeeder为可开放容量最小线路，属性feederName为线路名称、availCap为可开放容量数值
                SqliteDb sqliteDb = new SqliteDb(args[1]);
                MaxMinAvailCap maxFeeder = sqliteDb.queryMaxMinAvailCap(maxMinAvailCapTable, 1);
                MaxMinAvailCap minFeeder = sqliteDb.queryMaxMinAvailCap(maxMinAvailCapTable, 2);
                System.out.println(maxFeeder.feederName + " " + maxFeeder.availCap);
                System.out.println(minFeeder.feederName + " " + minFeeder.availCap);
                break;
            }
            case "baseInfo": {
                // baseInfo为开关基础信息，查询线路限额，最大电流，平均电流。args[1]为馈线数据库文件夹的路径，args[2]为馈线名称，args[3]为开关mRID
                // lineRatedI为最大限额，lineMaxI为最大电流，avgI为平均电流
                SqliteDb sqliteDb = new SqliteDb(args[1] + "\\" + args[2] + ".db");
                double lineRatedI = sqliteDb.queryOneLineParam(args[2] + oneLineParamTableName, args[3]);
                double lineMaxI = sqliteDb.queryMaxSwitchI(args[2] + switchTableName + HistoryData.seasonTable, args[3], -1);
                double avgI = sqliteDb.queryMaxSwitchI(args[2] + switchTableName + HistoryData.seasonTable, args[3], -2);
                System.out.println(lineRatedI + " " + lineMaxI + " " + avgI);
                break;
            }
            case "lineSummer": {
                // lineSummer为线路夏季分析。args[1]为馈线数据库文件夹的路径，args[2]为馈线名称，args[3]为开关mRID
                // lineRatedICv为线路限额，seasonCluster2为负荷聚类曲线，seasonMax2为最大负荷曲线，cap2为可开放容量
                SqliteDb sqliteDb = new SqliteDb(args[1] + "\\" + args[2] + ".db");
                double lineRatedI = sqliteDb.queryOneLineSeasonParam(args[2] + oneLineParamTableName, args[3], 1);
                double[] lineRatedICv = new double[96];
                for (int i = 0; i < 96; i++) {
                    lineRatedICv[i] = lineRatedI;
                }
                double[] seasonCluster2 = sqliteDb.querySeasonSwitchI(args[2] + switchTableName + HistoryData.seasonClusterTable, args[3], 3, 96);
                double[] seasonMax2 = sqliteDb.querySeasonSwitchI(args[2] + switchTableName + HistoryData.seasonTable, args[3], 3, 96);
                double[] cap2 = sqliteDb.queryAvailCap(args[2] + availCapTableName, args[3], 3, 96);
//                System.out.println("时段,1,2,3,4");
//                for (int i = 0; i < 24; i++) {
//                    System.out.println(i + "," + cap1[4 * i] + "," + cap2[4 * i] + "," + cap3[4 * i] + "," + cap4[4 * i]);
//                }
//                System.out.println();
                break;
            }
            case "lineWinter": {
                // lineWinter为线路冬季分析。args[1]为馈线数据库文件夹的路径，args[2]为馈线名称，args[3]为开关mRID
                // lineRatedICv为线路限额，seasonCluster4为负荷聚类曲线，seasonMax4为最大负荷曲线，cap4为可开放容量
                SqliteDb sqliteDb = new SqliteDb(args[1] + "\\" + args[2] + ".db");
                double lineRatedI = sqliteDb.queryOneLineSeasonParam(args[2] + oneLineParamTableName, args[3], 2);
                double[] lineRatedICv = new double[96];
                for (int i = 0; i < 96; i++) {
                    lineRatedICv[i] = lineRatedI;
                }
                double[] seasonCluster4 = sqliteDb.querySeasonSwitchI(args[2] + switchTableName + HistoryData.seasonClusterTable, args[3], 1, 96);
                double[] seasonMax4 = sqliteDb.querySeasonSwitchI(args[2] + switchTableName + HistoryData.seasonTable, args[3], 1, 96);
                double[] cap4 = sqliteDb.queryAvailCap(args[2] + availCapTableName, args[3], 1, 96);
                System.out.println();
                break;
            }
            case "linePs": {
                // linePs为线路峰谷分析。args[1]为馈线数据库文件夹的路径，args[2]为馈线名称，args[3]为开关mRID
                // lineRatedICv为线路限额，psCluster2为负荷聚类曲线，seasonMax2为最大负荷曲线，cap2为可开放容量
                SqliteDb sqliteDb = new SqliteDb(args[1] + "\\" + args[2] + ".db");
                double lineRatedI = sqliteDb.queryOneLineParam(args[2] + oneLineParamTableName, args[3]);
                double[] lineRatedICv = new double[96];
                for (int i = 0; i < 96; i++) {
                    lineRatedICv[i] = lineRatedI;
                }
                double[] psCluster2 = sqliteDb.querySeasonSwitchI(args[2] + switchTableName + HistoryData.psClusterTable, args[3], 2, 96);
                double[] seasonMax2 = sqliteDb.querySeasonSwitchI(args[2] + switchTableName + HistoryData.seasonTable, args[3], 2, 96);
                double[] cap2 = sqliteDb.queryAvailCap(args[2] + availCapTableName, args[3], 2, 96);
                System.out.println();
                break;
            }
            case "loadPosSeason": {
                // loadPosSeason为查询负荷接入分析结果。args[1]为存储所有馈线数据的数据库文件的路径
                // loadPosSeason为负荷接入分析结果列表，属性substation;feederName;canIn;tfName;phase;time对应界面表格的
                // 变电站、线路名称、能否接入、接入点、接入相别、分析时间
                SqliteDb sqliteDb = new SqliteDb(args[1]);
                List<LoadPosSeason> loadPosSeason = sqliteDb.queryLoadPosSeason(loadPosTable, 1,-1,-1);
                System.out.println(loadPosSeason.size());
                break;
            }
            case "loadPosSw": {
                // loadPosSw为查询高压情况负荷接入分析结果。args[1]为存储所有馈线数据的数据库文件的路径，args[2]为loadId（从loadPosSeason属性获取）
                // loadPosSw为高压情况负荷接入分析结果，属性canIn为是否能接入，swName为接入点智能开关名称，newLoadI为接入负荷电流曲线，
                // swOrgLoad为接入前接入点负荷电流曲线，swNewLoad为接入后负荷电流曲线，swRateI为限额
                SqliteDb sqliteDb = new SqliteDb(args[1]);
                LoadPosSw loadPosSw = sqliteDb.queryLoadPosSw(loadPosTable, Integer.parseInt(args[2]), 1);
                System.out.println();
                break;
            }
            case "loadPosTf": {
                // loadPosTf为查询低压情况负荷接入分析结果。args[1]为数据库文件的路径，args[2]为loadId（从loadPosSeason属性获取）
                // loadPosTf为低压情况负荷接入分析结果，属性canIn为是否能接入，tfName为接入点，phase为接入相别，newLoad为接入负荷电流曲线，
                // tfOrgLoad为接入前接入点负荷电流曲线，tfNewLoad为接入后负荷电流曲线，tfRateCap为限额
                SqliteDb sqliteDb = new SqliteDb(args[1]);
                LoadPosTf loadPosTf = sqliteDb.queryLoadPosTf(loadPosTable, Integer.parseInt(args[2]), 1);
                System.out.println();
                break;
            }
            case "feederWarnDev": {
                // feederWarnDev为查询预警元件颜色。args[1]为馈线数据库文件夹路径，args[2]为查询的馈线名称
                // yellowTfs为显示黄色的公变列表，redTfs为显示红色的公变列表，yellowLines为显示黄色的线路列表，redLines为显示红色的线路列表，
                // 属性loadState为1表示黄色，为2表示红色，属性mRID为设备mRID
                SqliteDb sqliteDb = new SqliteDb(args[1] + "\\" + args[2] + ".db");
                List<WarnTf> warnTfs = sqliteDb.queryWarnTf(args[2] + tfWarnTableName);
                List<WarnLine> warnLines = sqliteDb.queryWarnLine(args[2] + lineWarnTableName);
                List<WarnTf> yellowTfs = new LinkedList<>();    // 显示黄色的公变
                List<WarnTf> redTfs = new LinkedList<>();   // 显示红色的公变
                List<WarnLine> yellowLines = new LinkedList<>();   // 显示黄色的线路
                List<WarnLine> redLines = new LinkedList<>();    // 显示红色的线路
                for (WarnTf warnTf : warnTfs) {
                    if (warnTf.loadState == 1)
                        yellowTfs.add(warnTf);
                    else
                        redTfs.add(warnTf);
                }
                for (WarnLine warnLine : warnLines) {
                    if (warnLine.loadState == 1)
                        yellowLines.add(warnLine);
                    else
                        redLines.add(warnLine);
                }
                System.out.println(yellowTfs.size() + ", " + redTfs.size() + ", " + yellowLines.size() + ", " + redLines.size());
                break;
            }
        }
    }
}
