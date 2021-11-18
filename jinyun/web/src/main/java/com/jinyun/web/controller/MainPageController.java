package com.jinyun.web.controller;

import com.jinyun.web.service.CapService;
import io.swagger.annotations.Api;
import io.swagger.annotations.ApiOperation;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.*;

import java.util.List;
import java.util.Map;

@Api(value = "desc of class", tags="MainPage",description = "首页")
@RestController
@RequestMapping("mainPage")
public class MainPageController {
    @Autowired
    CapService capService;

    @ApiOperation(value = "电网统计字段", notes = "")
    @RequestMapping(value = "/gridStatistics",method = RequestMethod.GET)
    public Object gridStatistics() {
        Map<String,Object> result = capService.gridStatistics();
        return result;
    }

    @ApiOperation(value = "低裕度预警统计", notes = "")
    @RequestMapping(value = "/lowMarginWarnStatistics",method = RequestMethod.GET)
    public Object lowMarginWarnStatistics() {
        Map<String,Object> result = capService.lowMarginWarnStatistics();
        return result;
    }

    @ApiOperation(value = "重载线路列表", notes = "")
    @RequestMapping(value = "/hardLineList",method = RequestMethod.GET)
    public Object hardLineList() {
        List result = capService.hardLineList();
        return result;
    }

    @ApiOperation(value = "重载配变列表", notes = "")
    @RequestMapping(value = "/hardTransformerList",method = RequestMethod.GET)
    public Object hardTransformerList() {
        List result = capService.hardTransformerList();
        return result;
    }

    @ApiOperation(value = "超载线路列表", notes = "")
    @RequestMapping(value = "/overLineList",method = RequestMethod.GET)
    public Object overLineList() {
        List result = capService.overLineList();
        return result;
    }

    @ApiOperation(value = "超载配变列表", notes = "")
    @RequestMapping(value = "/overTransformerList",method = RequestMethod.GET)
    public Object overTransformerList() {
        List result = capService.overTransformerList();
        return result;
    }


    @ApiOperation(value = "配变三相不平衡度", notes = "")
    @RequestMapping(value = "/transformerUnbalance",method = RequestMethod.GET)
    public Object transformerUnbalance() {
        Map<String,Object> result = capService.transformerUnbalance();
        return result;
    }

    @ApiOperation(value = "配变三相不平衡度列表", notes = "")
    @RequestMapping(value = "/transformerUnbalanceList",method = RequestMethod.GET)
    public Object transformerUnbalanceList() {
        List result = capService.transformerUnbalanceList();
        return result;
    }

    @ApiOperation(value = "线路电流合格率", notes = "")
    @RequestMapping(value = "/linePassRate",method = RequestMethod.GET)
    public Object linePassRate() {
        List result = capService.linePassRate();
        return result;
    }

    @ApiOperation(value = "配变负荷越限统计", notes = "")
    @RequestMapping(value = "/transformerOverLoadRate",method = RequestMethod.GET)
    public Object transformerOverLoadRate() {
        Object result = capService.transformerOverLoadRate();
        return result;
    }

    @ApiOperation(value = "线路可开放容量大小", notes = "")
    @RequestMapping(value = "/maxMinAvailCap",method = RequestMethod.GET)
    public Object maxMinAvailCap() {
        Map<String, Object> result = capService.maxMinAvailCap();
        return result;
    }
}
