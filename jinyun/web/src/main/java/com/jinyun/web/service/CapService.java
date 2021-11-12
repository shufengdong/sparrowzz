package com.jinyun.web.service;

import com.jinyun.cap.SqliteDb;
import com.jinyun.cap.TfUb;
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
}
